use std::{fs::File, io::Read};
use std::collections::VecDeque;

/// Result of exploring some file (lib.rs)
#[derive(PartialEq, Debug)]
pub enum Exploration {
	/// Source has unconditional #![no_std]
	NoStd,
	/// Source has conditional no-std feature, like #![cfg_attr(not(feature = "std"), no_std)]
	/// at the moment, only strict condition that it has to depend on "(not(feature = "std"))"
	Conditional,
	/// No notion of no-std
	Std,
}

struct MetaNestedIterator {
	values: VecDeque<syn::Meta>,
}

impl MetaNestedIterator {
	fn new(meta: syn::Meta) -> Self {
		let mut values = VecDeque::new();
		values.push_back(meta);

		Self { values }
	}
}

fn path_or_unnamed(path: &syn::Path) -> String {
	path.get_ident().map(|x| x.to_string()).unwrap_or("unnamed".to_string())
}

impl Iterator for MetaNestedIterator {
	type Item = String;

	fn next(&mut self) -> Option<String> {
		match self.values.pop_front() {
			Some(syn::Meta::List(meta_list)) => {
				let val = path_or_unnamed(&meta_list.path);
				for nested_meta in meta_list.nested.iter() {
					match nested_meta {
						syn::NestedMeta::Lit(_) => { continue; }
						syn::NestedMeta::Meta(m) => self.values.push_back(m.clone()),
					}
				}

				Some(val)
			},
			Some(syn::Meta::NameValue(name_value)) => {
				use quote::ToTokens;
				let mut val = path_or_unnamed(&name_value.path);
				val.push_str("=");
				val.push_str(&name_value.lit.to_token_stream().to_string());
				Some(val)
			},
			Some(syn::Meta::Path(path)) => {
				Some(path_or_unnamed(&path))
			}
			_ => None,
		}
	}
}

pub fn explore<R: AsRef<std::path::Path>>(file_path: R) -> Exploration {
	let mut file = File::open(file_path.as_ref()).expect("Unable to open file");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");

	let syn_file = syn::parse_file(&src).expect("Unable to parse file");

	for attr in syn_file.attrs.iter() {
		if let syn::AttrStyle::Inner(_) = attr.style {
			if attr.path.is_ident("no_std") {
				return Exploration::NoStd;
			} else if attr.path.is_ident("cfg_attr") {
				match attr.parse_meta() {
					Ok(meta) => {
						let values = MetaNestedIterator::new(meta).collect::<Vec<String>>();
						if &values == &["cfg_attr", "not", "no_std", "feature=\"std\""] {
							return Exploration::Conditional;
						}
					},
					_ => {
						continue;
					}
				}
			}
		}
	}

	Exploration::Std
 }

 #[cfg(test)]
 mod tests {

	#[test]
	fn trivia() {
		assert_eq!(super::explore("./res/test_unconditional.rs"), super::Exploration::NoStd);
		assert_eq!(super::explore("./res/test_conditional.rs"), super::Exploration::Conditional);
		assert_eq!(super::explore("./res/test_std.rs"), super::Exploration::Std);
	}

 }