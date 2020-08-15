use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Peek};
use syn::{parenthesized, Attribute, Ident, LitInt, LitStr, Token};

use std::convert::TryFrom;

use crate::Spanned;
use proc_macro2::Span;
use sana_core::regex::Regex;

struct RegexExpr(Regex);

pub(crate) fn parse_backend_attr(attr: Attribute) -> Option<crate::Backend> {
    let name = attr.path.get_ident()?.to_string();
    if &*name != "backend" {
        return None;
    }

    let ba: BackendAttr = syn::parse2(attr.tokens).map_err(|e| emit_error!(e)).ok()?;

    Some(ba.0)
}

pub(crate) fn parse_pprint_ir_attr(attr: Attribute) -> Option<Span> {
    let name = attr.path.get_ident()?.to_string();
    if &*name == "pprint_ir" {
        Some(attr.bracket_token.span)
    } else {
        None
    }
}

struct BackendAttr(crate::Backend);

impl Parse for BackendAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let backend: Ident = content.parse()?;

        match &*backend.to_string() {
            "vm" => Ok(BackendAttr(crate::Backend::Vm)),
            "rust" => Ok(BackendAttr(crate::Backend::Rust)),
            _ => Err(input.error("Invalid backend")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SanaAttr {
    Regex(RegexAttr),
    Token(TokenAttr),
    Error,
}

pub(crate) fn parse_attr(attr: Attribute) -> Option<Spanned<SanaAttr>> {
    let name = attr.path.get_ident()?.to_string();
    let data = match &*name {
        "regex" => SanaAttr::Regex(syn::parse2(attr.tokens).map_err(|e| emit_error!(e)).ok()?),
        "token" => SanaAttr::Token(syn::parse2(attr.tokens).map_err(|e| emit_error!(e)).ok()?),
        "error" => SanaAttr::Error,
        _ => return None,
    };

    Some(Spanned {
        data,
        span: attr.bracket_token.span,
    })
}

fn parse_infix<Any, T, Op, F>(
    input: ParseStream,
    op: Op,
    cons: F,
    higher: fn(ParseStream) -> syn::Result<Regex>,
) -> syn::Result<Regex>
where
    T: Parse,
    Op: Copy + Peek + FnOnce(Any) -> T,
    F: FnOnce(Vec<Regex>) -> Regex,
{
    let head = input.call(higher)?;

    let mut tail = vec![];
    while input.peek(op) {
        input.parse::<T>()?;

        let expr = input.call(higher)?;
        tail.push(expr);
    }

    if tail.is_empty() {
        Ok(head)
    } else {
        Ok(cons(Some(head).into_iter().chain(tail).collect()))
    }
}

fn parse_regex_or(input: ParseStream) -> syn::Result<Regex> {
    parse_infix(input, Token![|], Regex::Or, parse_regex_and)
}

fn parse_regex_and(input: ParseStream) -> syn::Result<Regex> {
    parse_infix(input, Token![&], Regex::And, parse_regex_dot)
}

fn parse_regex_dot(input: ParseStream) -> syn::Result<Regex> {
    parse_infix(input, Token![.], Regex::Concat, parse_regex_not)
}

fn parse_regex_not(input: ParseStream) -> syn::Result<Regex> {
    if input.peek(Token![!]) {
        input.parse::<Token![!]>()?;
        let inner = parse_regex_not(input)?;

        Ok(Regex::Not(Box::new(inner)))
    } else {
        parse_regex_other(input)
    }
}

fn parse_regex_other(input: ParseStream) -> syn::Result<Regex> {
    if input.peek(syn::LitStr) {
        let regex: LitStr = input.parse()?;
        let span = regex.span();
        let hir = regex_syntax::Parser::new()
            .parse(&regex.value())
            .map_err(|e| syn::Error::new(span, e))?;
        let regex = Regex::try_from(hir).map_err(|e| syn::Error::new(span, e))?;

        Ok(regex)
    } else {
        let content;
        parenthesized!(content in input);

        parse_regex_or(&content)
    }
}

impl Parse for RegexExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() || input.peek(Token![,]) {
            return Err(input.error("Empty regex"));
        }

        let regex = parse_regex_or(input)?;

        Ok(RegexExpr(regex))
    }
}

struct KeyValue {
    key: Ident,
    value: Value,
}

enum Value {
    Int(LitInt),
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;

        input.parse::<Token![=]>()?;

        let value = if input.peek(LitInt) {
            Value::Int(input.parse()?)
        } else {
            return Err(input.error("Invalid value"));
        };

        Ok(KeyValue { key, value })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegexAttr {
    pub regex: Regex,
    pub priority: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAttr {
    pub token: Regex,
    pub priority: usize,
}

impl Parse for RegexAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let input = content;

        let regex = input.parse::<RegexExpr>()?.0;

        if input.is_empty() {
            return Ok(RegexAttr { regex, priority: 0 });
        } else {
            input.parse::<Token![,]>()?;
        }

        let mut priority = 0;

        let kvs = input.parse_terminated::<_, Token![,]>(KeyValue::parse)?;
        for kv in kvs {
            match &*kv.key.to_string() {
                "priority" => {
                    let Value::Int(value) = kv.value;

                    priority = value.base10_parse()?;
                }
                _ => return Err(syn::Error::new(kv.key.span(), "Invalid parameter name")),
            }
        }

        Ok(RegexAttr { regex, priority })
    }
}

impl Parse for TokenAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let input = content;

        let token = input.parse::<LitStr>()?;
        let token = Regex::literal_str(&token.value());

        if input.is_empty() {
            return Ok(TokenAttr { token, priority: 0 });
        } else {
            input.parse::<Token![,]>()?;
        }

        let mut priority = 0;

        let kvs = input.parse_terminated::<_, Token![,]>(KeyValue::parse)?;
        for kv in kvs {
            match &*kv.key.to_string() {
                "priority" => {
                    let Value::Int(value) = kv.value;

                    priority = value.base10_parse()?;
                }
                _ => return Err(syn::Error::new(kv.key.span(), "Invalid parameter name")),
            }
        }

        Ok(TokenAttr { token, priority })
    }
}
