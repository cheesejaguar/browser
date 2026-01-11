//! CSS Parser.

use crate::media::{MediaQuery, MediaQueryList};
use crate::properties::{PropertyDeclaration, PropertyId};
use crate::selector::{Selector, SelectorList};
use crate::stylesheet::{
    CssRule, FontFaceRule, ImportRule, KeyframeRule, KeyframesRule, MediaRule, StyleRule,
    Stylesheet, SupportsRule,
};
use crate::values::CssValue;
use cssparser::{
    BasicParseError, BasicParseErrorKind, CowRcStr, DeclarationParser,
    ParseError, Parser, ParserInput, ParserState, RuleBodyItemParser, RuleBodyParser,
    StyleSheetParser, ToCss, Token,
};
use std::sync::Arc;
use url::Url;

/// CSS Parser.
pub struct CssParser {
    base_url: Url,
}

impl CssParser {
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    /// Parse a CSS stylesheet.
    pub fn parse(&self, css: &str) -> Stylesheet {
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);

        let mut stylesheet = Stylesheet::new(self.base_url.clone());

        let mut rule_parser = TopLevelRuleParser {
            base_url: &self.base_url,
        };

        for result in StyleSheetParser::new(&mut parser, &mut rule_parser) {
            match result {
                Ok(rule) => stylesheet.rules.push(rule),
                Err((err, _slice)) => {
                    tracing::warn!("CSS parse error: {:?}", err);
                }
            }
        }

        stylesheet
    }

    /// Parse a style attribute value.
    pub fn parse_style_attribute(&self, css: &str) -> Vec<PropertyDeclaration> {
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);

        let mut decl_parser = StyleAttributeParser {
            base_url: &self.base_url,
        };

        let body_parser = RuleBodyParser::new(&mut parser, &mut decl_parser);

        body_parser
            .filter_map(|result| result.ok())
            .collect()
    }
}

/// Parse CSS stylesheet from string.
pub fn parse_css(css: &str, base_url: Url) -> Stylesheet {
    CssParser::new(base_url).parse(css)
}

/// Parse style attribute.
pub fn parse_style_attribute(css: &str) -> Vec<PropertyDeclaration> {
    CssParser::new(Url::parse("about:blank").unwrap()).parse_style_attribute(css)
}

/// Top-level rule parser.
struct TopLevelRuleParser<'a> {
    base_url: &'a Url,
}

impl<'i> cssparser::QualifiedRuleParser<'i> for TopLevelRuleParser<'_> {
    type Prelude = SelectorList;
    type QualifiedRule = CssRule;
    type Error = CssParseError<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        parse_selector_list(input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let declarations = parse_declaration_block(input, self.base_url);
        Ok(CssRule::Style(StyleRule {
            selectors: prelude,
            declarations,
        }))
    }
}

impl<'i> cssparser::AtRuleParser<'i> for TopLevelRuleParser<'_> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;
    type Error = CssParseError<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        match &*name {
            "import" => {
                let url = input.expect_url_or_string()?.as_ref().to_string();
                let media = parse_media_query_list(input);
                Ok(AtRulePrelude::Import(url, media))
            }
            "media" => {
                let media = parse_media_query_list(input);
                Ok(AtRulePrelude::Media(media))
            }
            "font-face" => Ok(AtRulePrelude::FontFace),
            "keyframes" | "-webkit-keyframes" => {
                let name = input.expect_ident_or_string()?.as_ref().to_string();
                Ok(AtRulePrelude::Keyframes(name))
            }
            "supports" => {
                let condition = parse_supports_condition(input)?;
                Ok(AtRulePrelude::Supports(condition))
            }
            "charset" => {
                let _encoding = input.expect_string()?.as_ref().to_string();
                Ok(AtRulePrelude::Charset)
            }
            "namespace" => {
                let prefix = input.try_parse(|i| i.expect_ident().map(|s| s.as_ref().to_string())).ok();
                let url = input.expect_url_or_string()?.as_ref().to_string();
                Ok(AtRulePrelude::Namespace(prefix, url))
            }
            "page" => {
                let selector = input.try_parse(|i| i.expect_ident().map(|s| s.as_ref().to_string())).ok();
                Ok(AtRulePrelude::Page(selector))
            }
            _ => Err(input.new_custom_error(CssParseError::UnknownAtRule(name.to_string()))),
        }
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        match prelude {
            AtRulePrelude::Import(url, media) => {
                let resolved_url = self.base_url.join(&url).ok();
                Ok(CssRule::Import(ImportRule {
                    url,
                    resolved_url,
                    media,
                    stylesheet: None,
                }))
            }
            AtRulePrelude::Charset => Ok(CssRule::Charset),
            AtRulePrelude::Namespace(prefix, url) => {
                Ok(CssRule::Namespace { prefix, url })
            }
            _ => Err(()),
        }
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        match prelude {
            AtRulePrelude::Media(media) => {
                let rules = parse_rule_list(input, self.base_url);
                Ok(CssRule::Media(MediaRule { media, rules }))
            }
            AtRulePrelude::FontFace => {
                let declarations = parse_declaration_block(input, self.base_url);
                Ok(CssRule::FontFace(FontFaceRule { declarations }))
            }
            AtRulePrelude::Keyframes(name) => {
                let keyframes = parse_keyframes(input, self.base_url);
                Ok(CssRule::Keyframes(KeyframesRule { name, keyframes }))
            }
            AtRulePrelude::Supports(condition) => {
                let rules = parse_rule_list(input, self.base_url);
                Ok(CssRule::Supports(SupportsRule { condition, rules }))
            }
            AtRulePrelude::Page(selector) => {
                let declarations = parse_declaration_block(input, self.base_url);
                Ok(CssRule::Page { selector, declarations })
            }
            _ => Err(input.new_custom_error(CssParseError::InvalidAtRule)),
        }
    }
}

/// At-rule prelude variants.
enum AtRulePrelude {
    Import(String, MediaQueryList),
    Media(MediaQueryList),
    FontFace,
    Keyframes(String),
    Supports(String),
    Charset,
    Namespace(Option<String>, String),
    Page(Option<String>),
}

/// Style attribute parser (for parsing inline styles).
struct StyleAttributeParser<'a> {
    base_url: &'a Url,
}

impl<'i> DeclarationParser<'i> for StyleAttributeParser<'_> {
    type Declaration = PropertyDeclaration;
    type Error = CssParseError<'i>;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let property_id = PropertyId::from_name(&name);
        let value = parse_property_value(input, &property_id, self.base_url)?;
        let important = input
            .try_parse(|i| {
                i.expect_delim('!')?;
                i.expect_ident_matching("important")
            })
            .is_ok();

        Ok(PropertyDeclaration {
            property: property_id,
            value,
            important,
        })
    }
}

impl<'i> cssparser::AtRuleParser<'i> for StyleAttributeParser<'_> {
    type Prelude = ();
    type AtRule = PropertyDeclaration;
    type Error = CssParseError<'i>;
}

impl<'i> cssparser::QualifiedRuleParser<'i> for StyleAttributeParser<'_> {
    type Prelude = ();
    type QualifiedRule = PropertyDeclaration;
    type Error = CssParseError<'i>;
}

impl<'i> RuleBodyItemParser<'i, PropertyDeclaration, CssParseError<'i>> for StyleAttributeParser<'_> {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}

/// Property declaration parser for declaration blocks.
struct PropertyDeclarationParser<'a> {
    base_url: &'a Url,
}

impl<'i> DeclarationParser<'i> for PropertyDeclarationParser<'_> {
    type Declaration = PropertyDeclaration;
    type Error = CssParseError<'i>;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let property_id = PropertyId::from_name(&name);

        // Parse the value
        let value = parse_property_value(input, &property_id, self.base_url)?;

        // Check for !important
        let important = input
            .try_parse(|i| {
                i.expect_delim('!')?;
                i.expect_ident_matching("important")
            })
            .is_ok();

        Ok(PropertyDeclaration {
            property: property_id,
            value,
            important,
        })
    }
}

impl<'i> cssparser::AtRuleParser<'i> for PropertyDeclarationParser<'_> {
    type Prelude = ();
    type AtRule = PropertyDeclaration;
    type Error = CssParseError<'i>;
}

impl<'i> cssparser::QualifiedRuleParser<'i> for PropertyDeclarationParser<'_> {
    type Prelude = ();
    type QualifiedRule = PropertyDeclaration;
    type Error = CssParseError<'i>;
}

impl<'i> RuleBodyItemParser<'i, PropertyDeclaration, CssParseError<'i>> for PropertyDeclarationParser<'_> {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}

/// Custom parse error.
#[derive(Clone, Debug)]
pub enum CssParseError<'i> {
    UnknownAtRule(String),
    InvalidAtRule,
    InvalidSelector,
    InvalidValue,
    Basic(BasicParseErrorKind<'i>),
}

impl<'i> From<BasicParseError<'i>> for CssParseError<'i> {
    fn from(e: BasicParseError<'i>) -> Self {
        CssParseError::Basic(e.kind)
    }
}

/// Parse selector list.
fn parse_selector_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<SelectorList, ParseError<'i, CssParseError<'i>>> {
    let mut selectors = Vec::new();

    loop {
        let selector = parse_selector(input)?;
        selectors.push(selector);

        let state = input.state();
        match input.next() {
            Ok(&Token::Comma) => continue,
            Ok(_) => {
                input.reset(&state);
                break;
            }
            Err(_) => break,
        }
    }

    Ok(SelectorList { selectors })
}

/// Parse a single selector.
fn parse_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Selector, ParseError<'i, CssParseError<'i>>> {
    let mut selector = Selector::default();
    let mut has_content = false;

    loop {
        // Try to parse a simple selector component
        let state = input.state();
        match input.next_including_whitespace() {
            Ok(token) => match token {
                Token::Ident(ident) => {
                    if selector.tag.is_none() && !has_content {
                        selector.tag = Some(ident.to_string().to_ascii_lowercase());
                    } else {
                        input.reset(&state);
                        break;
                    }
                    has_content = true;
                }
                Token::IDHash(id) | Token::Hash(id) => {
                    selector.id = Some(id.to_string());
                    has_content = true;
                }
                Token::Delim('.') => {
                    let class = input.expect_ident()?.to_string();
                    selector.classes.push(class);
                    has_content = true;
                }
                Token::Delim('*') => {
                    selector.universal = true;
                    has_content = true;
                }
                Token::SquareBracketBlock => {
                    let attr = input.parse_nested_block(|input| {
                        parse_attribute_selector(input)
                    })?;
                    selector.attributes.push(attr);
                    has_content = true;
                }
                Token::Colon => {
                    // Pseudo-class or pseudo-element
                    if input.try_parse(|i| i.expect_colon()).is_ok() {
                        // Pseudo-element (::)
                        let name = input.expect_ident()?.to_string();
                        selector.pseudo_elements.push(name);
                    } else {
                        // Pseudo-class (:)
                        let name = input.expect_ident()?.to_string();

                        // Handle functional pseudo-classes
                        if input.try_parse(|i| i.expect_parenthesis_block()).is_ok() {
                            let args = input.parse_nested_block(|input| {
                                let mut args = String::new();
                                while let Ok(token) = input.next() {
                                    args.push_str(&token.to_css_string());
                                }
                                Ok::<_, ParseError<'_, CssParseError<'_>>>(args)
                            })?;
                            selector.pseudo_classes.push((name, Some(args)));
                        } else {
                            selector.pseudo_classes.push((name, None));
                        }
                    }
                    has_content = true;
                }
                Token::WhiteSpace(_) => {
                    if has_content {
                        // Descendant combinator
                        selector.combinator = Some(crate::selector::Combinator::Descendant);
                        break;
                    }
                }
                Token::Delim('>') => {
                    selector.combinator = Some(crate::selector::Combinator::Child);
                    break;
                }
                Token::Delim('+') => {
                    selector.combinator = Some(crate::selector::Combinator::NextSibling);
                    break;
                }
                Token::Delim('~') => {
                    selector.combinator = Some(crate::selector::Combinator::SubsequentSibling);
                    break;
                }
                Token::Comma | Token::CurlyBracketBlock => {
                    input.reset(&state);
                    break;
                }
                _ => {
                    input.reset(&state);
                    break;
                }
            },
            Err(_) => break,
        }
    }

    if !has_content {
        return Err(input.new_custom_error(CssParseError::InvalidSelector));
    }

    // Parse next part if there's a combinator
    if selector.combinator.is_some() {
        // Skip whitespace after combinator
        while input.try_parse(|i| i.expect_whitespace()).is_ok() {}

        if let Ok(next) = parse_selector(input) {
            selector.next = Some(Box::new(next));
        }
    }

    Ok(selector)
}

/// Parse attribute selector.
fn parse_attribute_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<crate::selector::AttributeSelector, ParseError<'i, CssParseError<'i>>> {
    let name = input.expect_ident()?.to_string();

    let (operator, value, case_sensitivity) = if input.is_exhausted() {
        (None, None, crate::selector::CaseSensitivity::Default)
    } else {
        let op = input.expect_delim('=').ok().map(|_| "=".to_string()).or_else(|| {
            input.try_parse(|i| -> Result<String, ParseError<'_, CssParseError<'_>>> {
                let c = match i.next()? {
                    Token::Delim(c) => *c,
                    _ => return Err(i.new_custom_error(CssParseError::InvalidSelector)),
                };
                i.expect_delim('=')?;
                Ok(format!("{}=", c))
            }).ok()
        });

        if let Some(op) = op {
            let value = input.expect_ident_or_string()?.as_ref().to_string();

            let case = input.try_parse(|i| -> Result<crate::selector::CaseSensitivity, ParseError<'_, CssParseError<'_>>> {
                let ident = i.expect_ident()?;
                match ident.as_ref() {
                    "i" | "I" => Ok(crate::selector::CaseSensitivity::Insensitive),
                    "s" | "S" => Ok(crate::selector::CaseSensitivity::Sensitive),
                    _ => Err(i.new_custom_error(CssParseError::InvalidSelector)),
                }
            }).unwrap_or(crate::selector::CaseSensitivity::Default);

            (Some(op), Some(value), case)
        } else {
            (None, None, crate::selector::CaseSensitivity::Default)
        }
    };

    Ok(crate::selector::AttributeSelector {
        name,
        operator,
        value,
        case_sensitivity,
    })
}

/// Parse declaration block.
fn parse_declaration_block(input: &mut Parser<'_, '_>, base_url: &Url) -> Vec<PropertyDeclaration> {
    let mut parser = PropertyDeclarationParser { base_url };
    let body_parser = RuleBodyParser::new(input, &mut parser);
    let mut declarations = Vec::new();
    for result in body_parser {
        if let Ok(decl) = result {
            declarations.push(decl);
        }
    }
    declarations
}

/// Parse rule list (for nested rules).
fn parse_rule_list(input: &mut Parser<'_, '_>, base_url: &Url) -> Vec<CssRule> {
    let mut rule_parser = TopLevelRuleParser { base_url };
    let list_parser = StyleSheetParser::new(input, &mut rule_parser);
    let mut rules = Vec::new();
    for result in list_parser {
        if let Ok(rule) = result {
            rules.push(rule);
        }
    }
    rules
}

/// Parse media query list.
fn parse_media_query_list(input: &mut Parser<'_, '_>) -> MediaQueryList {
    let mut queries = Vec::new();

    loop {
        if let Ok(query) = parse_media_query(input) {
            queries.push(query);
        }

        match input.next() {
            Ok(&Token::Comma) => continue,
            _ => break,
        }
    }

    MediaQueryList { queries }
}

/// Parse a single media query.
fn parse_media_query<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<MediaQuery, ParseError<'i, CssParseError<'i>>> {
    // Simple implementation - real one would be more complete
    let mut media_type = None;
    let mut features = Vec::new();
    let mut negated = false;

    // Check for "not" or "only"
    if input.try_parse(|i| i.expect_ident_matching("not")).is_ok() {
        negated = true;
    } else {
        input.try_parse(|i| i.expect_ident_matching("only")).ok();
    }

    // Media type
    if let Ok(ident) = input.try_parse(|i| i.expect_ident().map(|s| s.to_string())) {
        media_type = Some(crate::media::MediaType::from_str(&ident));
    }

    // Media features
    while input.try_parse(|i| i.expect_ident_matching("and")).is_ok() {
        if let Ok(feature) = input.try_parse(|i| {
            i.expect_parenthesis_block()?;
            i.parse_nested_block(parse_media_feature)
        }) {
            features.push(feature);
        }
    }

    Ok(MediaQuery {
        media_type,
        features,
        negated,
    })
}

/// Parse a media feature.
fn parse_media_feature<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<crate::media::MediaFeature, ParseError<'i, CssParseError<'i>>> {
    let name = input.expect_ident()?.to_string();

    let value = if input.try_parse(|i| i.expect_colon()).is_ok() {
        Some(parse_component_value(input)?)
    } else {
        None
    };

    Ok(crate::media::MediaFeature { name, value })
}

/// Parse supports condition.
fn parse_supports_condition<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<String, ParseError<'i, CssParseError<'i>>> {
    // Simple implementation - collect condition as string
    let mut condition = String::new();

    while let Ok(token) = input.next() {
        condition.push_str(&token.to_css_string());
    }

    Ok(condition)
}

/// Parse keyframes.
fn parse_keyframes(input: &mut Parser<'_, '_>, base_url: &Url) -> Vec<KeyframeRule> {
    let mut keyframes = Vec::new();

    while !input.is_exhausted() {
        // Parse keyframe selector
        let selectors = match parse_keyframe_selectors(input) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Parse block
        if input.expect_curly_bracket_block().is_err() {
            continue;
        }

        let declarations = input.parse_nested_block(|input| {
            Ok::<_, ParseError<'_, CssParseError<'_>>>(parse_declaration_block(input, base_url))
        }).unwrap_or_default();

        keyframes.push(KeyframeRule {
            selectors,
            declarations,
        });
    }

    keyframes
}

/// Parse keyframe selectors.
fn parse_keyframe_selectors<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Vec<String>, ParseError<'i, CssParseError<'i>>> {
    let mut selectors = Vec::new();

    loop {
        if let Ok(ident) = input.try_parse(|i| i.expect_ident().map(|s| s.to_string())) {
            selectors.push(ident);
        } else if let Ok(percentage) = input.try_parse(|i| i.expect_percentage()) {
            selectors.push(format!("{}%", percentage * 100.0));
        } else {
            break;
        }

        // Check for comma, but don't consume non-comma tokens
        if input.try_parse(|i| i.expect_comma()).is_err() {
            break;
        }
    }

    if selectors.is_empty() {
        Err(input.new_custom_error(CssParseError::InvalidSelector))
    } else {
        Ok(selectors)
    }
}

/// Parse property value.
fn parse_property_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    _property: &PropertyId,
    base_url: &Url,
) -> Result<CssValue, ParseError<'i, CssParseError<'i>>> {
    parse_component_value(input)
}

/// Parse a single component value.
fn parse_component_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssValue, ParseError<'i, CssParseError<'i>>> {
    let mut values = Vec::new();

    while let Ok(token) = input.next() {
        let value = match token {
            Token::Ident(s) => CssValue::Ident(s.to_string()),
            Token::Number { value, .. } => CssValue::Number(*value),
            Token::Percentage { unit_value, .. } => CssValue::Percentage(*unit_value * 100.0),
            Token::Dimension { value, unit, .. } => {
                CssValue::Dimension(*value, unit.to_string())
            }
            Token::QuotedString(s) => CssValue::String(s.to_string()),
            Token::Hash(s) | Token::IDHash(s) => {
                CssValue::Color(format!("#{}", s))
            }
            Token::Function(name) => {
                let name = name.to_string();
                let args = input.parse_nested_block(|input| {
                    let mut args = Vec::new();
                    while let Ok(v) = parse_component_value(input) {
                        args.push(v);
                        input.try_parse(|i| i.expect_comma()).ok();
                    }
                    Ok::<_, ParseError<'_, CssParseError<'_>>>(args)
                })?;
                CssValue::Function(name, args)
            }
            Token::Comma => continue,
            Token::Delim('/') => {
                values.push(CssValue::Operator("/".to_string()));
                continue;
            }
            Token::WhiteSpace(_) => continue,
            _ => continue,
        };
        values.push(value);
    }

    if values.is_empty() {
        Err(input.new_custom_error(CssParseError::InvalidValue))
    } else if values.len() == 1 {
        Ok(values.pop().unwrap())
    } else {
        Ok(CssValue::List(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_css() {
        let css = "div { color: red; }";
        let stylesheet = parse_css(css, Url::parse("about:blank").unwrap());
        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_style_attribute() {
        let style = "color: red; font-size: 16px;";
        let declarations = parse_style_attribute(style);
        assert_eq!(declarations.len(), 2);
    }

    #[test]
    fn test_parse_media_rule() {
        let css = "@media screen and (min-width: 768px) { div { color: blue; } }";
        let stylesheet = parse_css(css, Url::parse("about:blank").unwrap());
        assert!(!stylesheet.rules.is_empty());
    }
}
