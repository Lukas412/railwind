mod types;

use types::*;

use crate::class::Decl;
use crate::utils::{get_args, get_class_name};
use crate::warning::WarningType;

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref MARGIN: HashMap<&'static str, &'static str> =
        ron::from_str(include_str!("margin.ron")).unwrap();
    pub static ref PADDING: HashMap<&'static str, &'static str> =
        ron::from_str(include_str!("padding.ron")).unwrap();
    pub static ref SPACE_BETWEEN: HashMap<&'static str, &'static str> =
        ron::from_str(include_str!("space_between.ron")).unwrap();
}

#[derive(Debug)]
pub enum Spacing<'a> {
    Padding(Padding<'a>),
    Margin(Margin<'a>),
    SpaceBetween(SpaceBetween<'a>),
}

impl<'a> Spacing<'a> {
    pub fn new(value: &'a str) -> Result<Option<Self>, WarningType> {
        let class_name = get_class_name(value);
        let args = get_args(value)?;

        let spacing = if let Some(padding) = Padding::new(class_name, args) {
            Spacing::Padding(padding)
        } else if let Some(margin) = Margin::new(class_name, args) {
            Spacing::Margin(margin)
        } else {
            if let Some(sb) = SpaceBetween::new(class_name, args)? {
                Spacing::SpaceBetween(sb)
            } else {
                return Ok(None);
            }
        };

        Ok(Some(spacing))
    }

    pub fn to_decl(self) -> Result<Decl, WarningType> {
        match self {
            Spacing::Padding(s) => s.to_decl(),
            Spacing::Margin(s) => s.to_decl(),
            Spacing::SpaceBetween(s) => s.to_decl(),
        }
    }
}
