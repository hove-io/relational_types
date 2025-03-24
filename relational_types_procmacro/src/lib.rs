#![deny(missing_docs)]

//! Custom derive for GetCorresponding. See `relational_types` for the documentation.

#![recursion_limit = "128"]

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Lit, Meta,
    PathArguments, Type,
};

/// Generation of the `GetCorresponding` trait implementation.
#[proc_macro_derive(GetCorresponding, attributes(get_corresponding))]
pub fn get_corresponding(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = impl_get_corresponding(&ast);
    gen.into()
}

fn impl_get_corresponding(ast: &DeriveInput) -> proc_macro2::TokenStream {
    if let Data::Struct(DataStruct {
        fields: Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        let name = &ast.ident;
        let edges: Vec<_> = fields.named.iter().filter_map(to_edge).collect();
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

        quote! {
            pub trait GetCorresponding<T: Sized> {
                fn get_corresponding(&self, model: &#name) -> IdxSet<T>;
            }
            impl #name {
                pub fn get_corresponding<T, U>(&self, from: &IdxSet<T>) -> IdxSet<U>
                where
                    IdxSet<T>: GetCorresponding<U>
                {
                    from.get_corresponding(self)
                }
                pub fn get_corresponding_from_idx<T, U>(&self, from: Idx<T>) -> IdxSet<U>
                where
                    IdxSet<T>: GetCorresponding<U>
                {
                    self.get_corresponding(&Some(from).into_iter().collect())
                }
            }
            #(#edges_impls)*
        }
    } else {
        quote!()
    }
}

fn to_edge(field: &Field) -> Option<Edge> {
    let ident = field.ident.as_ref()?.to_string();
    let mut split = ident.split("_to_");
    let _from_collection = split.next()?;
    let _to_collection = split.next()?;
    if split.next().is_some() {
        return None;
    }

    let segment = match &field.ty {
        Type::Path(type_path) => type_path.path.segments.last(),
        _ => None,
    }?;

    let (from_ty, to_ty) = if let PathArguments::AngleBracketed(data) = &segment.arguments {
        match (data.args.first(), data.args.get(1), data.args.get(2)) {
            (
                Some(syn::GenericArgument::Type(from_ty)),
                Some(syn::GenericArgument::Type(to_ty)),
                None,
            ) => Some((from_ty, to_ty)),
            _ => None,
        }
    } else {
        None
    }?;

    let weight = field
        .attrs
        .iter()
        .filter_map(|attr| {
            if let Meta::NameValue(meta) = &attr.meta {
                if meta.path.is_ident("weight") {
                    if let Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            return lit_str.value().parse::<f64>().ok();
                        }
                    }
                }
            }
            None
        })
        .last()
        .unwrap_or(1.0);

    Some(Edge {
        ident,
        from: (*from_ty).clone(),
        to: (*to_ty).clone(),
        weight,
    })
}

fn make_edge_to_get_corresponding<'a>(
    name: &Ident,
    edges: &'a [Edge],
) -> HashMap<(&'a Type, &'a Type), proc_macro2::TokenStream> {
    let mut res = HashMap::new();
    for e in edges {
        let ident = Ident::new(&e.ident, proc_macro2::Span::call_site());
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

fn floyd_warshall(edges: &[Edge]) -> HashMap<(&Type, &Type), &Type> {
    let mut v = HashSet::<&Type>::new();
    let mut dist = HashMap::<(&Type, &Type), f64>::new();
    let mut next = HashMap::new();

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
            let dist_ik = *dist.get(&(i, k)).unwrap_or(&f64::INFINITY);
            for &j in &v {
                let dist_kj = *dist.get(&(k, j)).unwrap_or(&f64::INFINITY);
                let dist_ij = dist.entry((i, j)).or_insert(f64::INFINITY);
                if *dist_ij > dist_ik + dist_kj {
                    *dist_ij = dist_ik + dist_kj;
                    next.insert((i, j), next[&(i, k)]);
                }
            }
        }
    }

    next
}

struct Edge {
    ident: String,
    from: Type,
    to: Type,
    weight: f64,
}
