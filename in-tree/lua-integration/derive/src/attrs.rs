use syn::{Attribute, DeriveInput, Lit, Result, Error};

#[derive(Debug, Default)]
pub struct ContainerAttrs {
    pub serde: bool,
    pub eq: bool,
}

impl ContainerAttrs {
    pub fn from_derive_input(input: &DeriveInput) -> Result<Self> {
        let mut attrs = Self::default();

        for attr in &input.attrs {
            if !attr.path().is_ident("lua") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("serde") {
                    attrs.serde = true;
                    Ok(())
                } else if meta.path.is_ident("Eq") {
                    attrs.eq = true;
                    Ok(())
                } else {
                    Err(meta.error("unsupported lua attribute. Valid options: serde, Eq"))
                }
            })?;
        }

        Ok(attrs)
    }

}

#[derive(Debug, Default)]
pub struct VariantAttrs {
    pub rename: Option<String>,
}

impl VariantAttrs {
    pub fn from_attributes(attrs: &[Attribute]) -> Result<Self> {
        let mut variant_attrs = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("lua") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit_str) = s {
                        variant_attrs.rename = Some(lit_str.value());
                    } else {
                        return Err(Error::new_spanned(s, "rename value must be a string"));
                    }
                    Ok(())
                } else {
                    Err(meta.error("unsupported lua attribute"))
                }
            })?;
        }

        Ok(variant_attrs)
    }
}
