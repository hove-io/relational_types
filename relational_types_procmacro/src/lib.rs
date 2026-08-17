#![deny(missing_docs)]

//! Custom derive for GetCorresponding.  See `relational_types` for the documentation.

#![recursion_limit = "128"]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input};

/// Generation of the `GetCorresponding` trait implementation.
#[proc_macro_derive(GetCorresponding, attributes(get_corresponding))]
pub fn get_corresponding(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let result = match impl_get_corresponding(&ast) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    };
    result
}

fn impl_get_corresponding(ast: &DeriveInput) -> syn::Result<TokenStream2> {
    if let Data::Struct(ref data_struct) = ast.data {
        if let Fields::Named(ref named_fields) = data_struct.fields {
            let name = &ast.ident;
            let edges: Vec<_> = named_fields
                .named
                .iter()
                .filter_map(|f| to_edge(f).transpose())
                .collect::<syn::Result<_>>()?;
            let next = floyd_warshall(&edges);
            let edge_to_impl = make_edge_to_get_corresponding(name, &edges);
            let edges_impls = next.iter().map(|(&(from, to), &node)| {
                if from == to {
                    quote! {
                        impl GetCorresponding<#to> for IdxSet<#from> {
                            fn get_corresponding(&self, _: &#name) -> IdxSet<#to> {
                                self.clone()
                            }
                        }
                    }
                } else if to == node {
                    edge_to_impl[&(from, to)].clone()
                } else {
                    quote! {
                        impl GetCorresponding<#to> for IdxSet<#from> {
                            fn get_corresponding(&self, pt_objects: &#name) -> IdxSet<#to> {
                                let tmp: IdxSet<#node> = self.get_corresponding(pt_objects);
                                tmp.get_corresponding(pt_objects)
                            }
                        }
                    }
                }
            });
            return Ok(quote! {
                /// A trait that returns a set of objects corresponding to
                /// a given type.
                pub trait GetCorresponding<T: Sized> {
                    /// For the given self, returns the set of
                    /// corresponding `T` indices.
                    fn get_corresponding(&self, model: &#name) -> IdxSet<T>;
                }
                impl #name {
                    /// Returns the set of `U` indices corresponding to the `from` set.
                    pub fn get_corresponding<T, U>(&self, from: &IdxSet<T>) -> IdxSet<U>
                    where
                        IdxSet<T>: GetCorresponding<U>
                    {
                        from.get_corresponding(self)
                    }
                    /// Returns the set of `U` indices corresponding to the `from` index.
                    pub fn get_corresponding_from_idx<T, U>(&self, from: Idx<T>) -> IdxSet<U>
                    where
                        IdxSet<T>: GetCorresponding<U>
                    {
                        self.get_corresponding(&Some(from).into_iter().collect())
                    }
                }
                #(#edges_impls)*
            });
        }
    }
    Ok(quote!())
}

fn to_edge(field: &syn::Field) -> syn::Result<Option<Edge>> {
    let ident_str = match field.ident.as_ref() {
        Some(i) => i.to_string(),
        None => return Ok(None),
    };
    let parts: Vec<&str> = ident_str.split("_to_").collect();
    if parts.len() != 2 {
        return Ok(None);
    }
    let segment = if let Type::Path(ref type_path) = field.ty {
        type_path.path.segments.last()
    } else {
        None
    };
    let segment = match segment {
        Some(s) => s,
        None => return Ok(None),
    };
    let type_args = if let PathArguments::AngleBracketed(ref data) = segment.arguments {
        let types: Vec<&Type> = data
            .args
            .iter()
            .filter_map(|arg| {
                if let GenericArgument::Type(ty) = arg {
                    Some(ty)
                } else {
                    None
                }
            })
            .collect();
        match types.as_slice() {
            [from_ty, to_ty] => Some((*from_ty, *to_ty)),
            _ => None,
        }
    } else {
        None
    };
    let (from_ty, to_ty) = match type_args {
        Some(pair) => pair,
        None => return Ok(None),
    };

    let mut weight = 1.0f64;
    for attr in &field.attrs {
        if attr.path().is_ident("get_corresponding") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("weight") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    weight = s.value().parse::<f64>().map_err(|e| {
                        meta.error(format!(
                            "`weight` attribute must be convertible to f64: {e}"
                        ))
                    })?;
                    Ok(())
                } else {
                    Err(meta.error("Only `key = \"value\"` attributes supported."))
                }
            })?;
        }
    }

    Ok(Some(Edge {
        ident: ident_str,
        from: from_ty.clone(),
        to: to_ty.clone(),
        weight,
    }))
}

fn make_edge_to_get_corresponding<'a>(
    name: &syn::Ident,
    edges: &'a [Edge],
) -> HashMap<(&'a Type, &'a Type), TokenStream2> {
    let mut res = HashMap::default();
    for e in edges {
        let ident = Ident::new(&e.ident, Span::call_site());
        let from = &e.from;
        let to = &e.to;
        res.insert(
            (from, to),
            quote! {
                impl GetCorresponding<#to> for IdxSet<#from> {
                    fn get_corresponding(&self, pt_objects: &#name) -> IdxSet<#to> {
                        pt_objects.#ident.get_corresponding_forward(self)
                    }
                }
            },
        );
        res.insert(
            (to, from),
            quote! {
                impl GetCorresponding<#from> for IdxSet<#to> {
                    fn get_corresponding(&self, pt_objects: &#name) -> IdxSet<#from> {
                        pt_objects.#ident.get_corresponding_backward(self)
                    }
                }
            },
        );
    }
    res
}

fn floyd_warshall(edges: &[Edge]) -> HashMap<(&Node, &Node), &Node> {
    use std::f64::INFINITY;
    let mut v = HashSet::<&Node>::default();
    let mut dist = HashMap::<(&Node, &Node), f64>::default();
    let mut next = HashMap::default();
    for e in edges {
        let from = &e.from;
        let to = &e.to;
        v.insert(from);
        v.insert(to);
        dist.insert((from, to), e.weight);
        dist.insert((to, from), e.weight);
        next.insert((from, to), to);
        next.insert((to, from), from);
    }
    for &k in &v {
        for &i in &v {
            let dist_ik = match dist.get(&(i, k)) {
                Some(d) => *d,
                None => continue,
            };
            for &j in &v {
                let dist_kj = match dist.get(&(k, j)) {
                    Some(d) => *d,
                    None => continue,
                };
                let dist_ij = dist.entry((i, j)).or_insert(INFINITY);
                if *dist_ij > dist_ik + dist_kj {
                    *dist_ij = dist_ik + dist_kj;
                    let next_ik = next[&(i, k)];
                    next.insert((i, j), next_ik);
                }
            }
        }
    }
    next
}

struct Edge {
    ident: String,
    from: Node,
    to: Node,
    weight: f64,
}

type Node = Type;
