/// Taken from `clap_derive`:
/// <https://github.com/clap-rs/clap/blob/74109e5c1aa44cc7cd42f850fd73d091235bed7c/clap_derive/src/utils/doc_comments.rs#L8-L50>
pub fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    // multiline comments (`/** ... */`) may have LFs (`\n`) in them,
    // we need to split so we could handle the lines correctly
    //
    // we also need to remove leading and trailing blank lines
    let lines: Vec<_> = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            // non #[doc = "..."] attributes are not our concern
            // we leave them for rustc to handle
            match &attr.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }),
                    ..
                }) => Some(s.value()),
                _ => None,
            }
        })
        // .skip_while(|s| is_blank(s))
        .flat_map(|s| {
            let lines = s
                .split('\n')
                .map(|s| {
                    // remove one leading space no matter what
                    let s = s.strip_prefix(' ').unwrap_or(s);
                    s.to_owned()
                })
                .collect::<Vec<_>>();
            lines
        })
        .collect();

    // while let Some(true) = lines.last().map(|s| is_blank(s)) {
    //     lines.pop();
    // }

    lines.join("\n")
}
