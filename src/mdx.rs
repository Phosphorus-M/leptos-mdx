use leptos::{
    component, html::ElementDescriptor, Children, Fragment, HtmlElement, IntoView, View,
};
use regex::Regex;
use std::collections::HashMap;
use tl::{HTMLTag, Node};

use crate::markdown::parse;

#[component]
/// Renders a markdown source into a Leptos component.
/// Custom components can be used in the markdown source.
pub fn Mdx(source: String, components: Components) -> impl IntoView {
    let (_fm, html) = parse(&source).expect("invalid mdx");
    // TODO: we could expose frontmatter in the context so components can use its value

    let dom = tl::parse(&html, tl::ParserOptions::default()).expect("invalid html");

    let mut root_views = vec![];
    for node_handle in dom.children() {
        let node = node_handle.get(dom.parser()).expect("not a node");
        root_views.push(process_element(node, dom.parser(), &components, true));
    }

    Fragment::new(root_views)
}

/// Props passed to a custom component.
pub struct MdxComponentProps {
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: HashMap<String, Option<String>>,
    pub children: Children,
}

/// A collection of custom components.
pub struct Components {
    components: HashMap<String, Box<dyn Fn(MdxComponentProps) -> View>>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a new custom component that won't receive any props.
    pub fn add<F, IV>(&mut self, name: String, component: F)
    where
        F: Fn() -> IV + 'static,
        IV: IntoView + 'static,
    {
        self.components
            .insert(name, Box::new(move |_| component().into_view()));
    }

    /// Register a new custom component that will receive props. The standardized
    /// MdxComponentsProps are converted to the props type of the component using the provided
    /// adapter.
    pub fn add_props<F, IV, Props, PropsFn>(
        &mut self,
        name: String,
        component: F,
        props_adapter: PropsFn,
    ) where
        F: Fn(Props) -> IV + 'static,
        IV: IntoView + 'static,
        PropsFn: Fn(MdxComponentProps) -> Props + 'static,
    {
        self.components.insert(
            name,
            Box::new(move |props| component(props_adapter(props)).into_view()),
        );
    }

    fn get(&self, name: &str) -> Option<&Box<dyn Fn(MdxComponentProps) -> View>> {
        self.components.get(name)
    }
}

pub fn process_element(
    el: &Node,
    parser: &tl::Parser,
    components: &Components,
    parse_new_lines: bool,
) -> View {
    match el {
        Node::Comment(_comment) => return ().into_view(),
        Node::Raw(raw) => {
            let text = String::from_utf8(raw.as_bytes().to_vec());

            if let Ok(t) = text {
                if parse_new_lines {
                    /*
                     * Replace new lines with <br /> only if they are preceded and followed by text.
                     * to avoid adding <br /> to empty lines.
                     */
                    let reg = Regex::new(r"(.+)\n(.+)").unwrap();

                    let t = reg.replace_all(&t, |caps: &regex::Captures| {
                        format!("{} <br /> {}", &caps[1], &caps[2])
                    }).to_string();

                    return t.into_view();
                }

                return t.into_view();
            } else {
                println!("error parsing raw text: {:?}", text);
                return ().into_view();
            }
        }
        Node::Tag(tag) => {
            let mut child_views = vec![];

            let nodes = tag.children();

            // Process children
            nodes.top().iter().for_each(|node_handle| {
                let node = node_handle.get(parser).expect("not a node");

                /*
                 * Inside code blocks we want to keep the new lines as they are.
                 */
                if tag.name().as_utf8_str() == "code" || tag.name().as_utf8_str() == "pre" {
                    child_views.push(process_element(node, parser, components, false));
                } else {
                    child_views.push(process_element(node, parser, components, parse_new_lines));
                }
            });

            let name_ref = tag.name().as_utf8_str();
            let name = name_ref.as_ref();

            // Custom elements
            if let Some(component) = components.get(name) {
                let attributes = tag.attributes();

                let classes = attributes.class_iter().map_or(Vec::new(), |class_list| {
                    class_list.map(|c| c.to_string()).collect()
                });

                let attributes_map = attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.map(|v| v.to_string())))
                    .collect();

                return component(MdxComponentProps {
                    id: attributes.id().map(|id| id.as_utf8_str().to_string()),
                    classes,
                    attributes: attributes_map,
                    children: Box::new(move || Fragment::new(child_views)),
                });
            }

            // HTML elements
            match name {
                "html" => html_element(&tag.clone(), child_views, leptos::html::html()),
                "base" => html_element(&tag.clone(), child_views, leptos::html::base()),
                "head" => html_element(&tag.clone(), child_views, leptos::html::head()),
                "link" => html_element(&tag.clone(), child_views, leptos::html::link()),
                "meta" => html_element(&tag.clone(), child_views, leptos::html::meta()),
                "style" => html_element(&tag.clone(), child_views, leptos::html::style()),
                "title" => html_element(&tag.clone(), child_views, leptos::html::title()),
                "body" => html_element(&tag.clone(), child_views, leptos::html::body()),
                "address" => html_element(&tag.clone(), child_views, leptos::html::address()),
                "article" => html_element(&tag.clone(), child_views, leptos::html::article()),
                "aside" => html_element(&tag.clone(), child_views, leptos::html::aside()),
                "footer" => html_element(&tag.clone(), child_views, leptos::html::footer()),
                "header" => html_element(&tag.clone(), child_views, leptos::html::header()),
                "hgroup" => html_element(&tag.clone(), child_views, leptos::html::hgroup()),
                "h1" => html_element(&tag.clone(), child_views, leptos::html::h1()),
                "h2" => html_element(&tag.clone(), child_views, leptos::html::h2()),
                "h3" => html_element(&tag.clone(), child_views, leptos::html::h3()),
                "h4" => html_element(&tag.clone(), child_views, leptos::html::h4()),
                "h5" => html_element(&tag.clone(), child_views, leptos::html::h5()),
                "h6" => html_element(&tag.clone(), child_views, leptos::html::h6()),
                "main" => html_element(&tag.clone(), child_views, leptos::html::main()),
                "nav" => html_element(&tag.clone(), child_views, leptos::html::nav()),
                "section" => html_element(&tag.clone(), child_views, leptos::html::section()),
                "blockquote" => html_element(&tag.clone(), child_views, leptos::html::blockquote()),
                "dd" => html_element(&tag.clone(), child_views, leptos::html::dd()),
                "div" => html_element(&tag.clone(), child_views, leptos::html::div()),
                "dl" => html_element(&tag.clone(), child_views, leptos::html::dl()),
                "dt" => html_element(&tag.clone(), child_views, leptos::html::dt()),
                "figcaption" => html_element(&tag.clone(), child_views, leptos::html::figcaption()),
                "figure" => html_element(&tag.clone(), child_views, leptos::html::figure()),
                "hr" => html_element(&tag.clone(), child_views, leptos::html::hr()),
                "li" => html_element(&tag.clone(), child_views, leptos::html::li()),
                "ol" => html_element(&tag.clone(), child_views, leptos::html::ol()),
                "p" => html_element(&tag.clone(), child_views, leptos::html::p()),
                "pre" => html_element(&tag.clone(), child_views, leptos::html::pre()),
                "ul" => html_element(&tag.clone(), child_views, leptos::html::ul()),
                "a" => html_element(&tag.clone(), child_views, leptos::html::a()),
                "abbr" => html_element(&tag.clone(), child_views, leptos::html::abbr()),
                "b" => html_element(&tag.clone(), child_views, leptos::html::b()),
                "bdi" => html_element(&tag.clone(), child_views, leptos::html::bdi()),
                "bdo" => html_element(&tag.clone(), child_views, leptos::html::bdo()),
                "br" => html_element(&tag.clone(), child_views, leptos::html::br()),
                "cite" => html_element(&tag.clone(), child_views, leptos::html::cite()),
                "code" => html_element(&tag.clone(), child_views, leptos::html::code()),
                "data" => html_element(&tag.clone(), child_views, leptos::html::data()),
                "dfn" => html_element(&tag.clone(), child_views, leptos::html::dfn()),
                "em" => html_element(&tag.clone(), child_views, leptos::html::em()),
                "i" => html_element(&tag.clone(), child_views, leptos::html::i()),
                "kbd" => html_element(&tag.clone(), child_views, leptos::html::kbd()),
                "mark" => html_element(&tag.clone(), child_views, leptos::html::mark()),
                "q" => html_element(&tag.clone(), child_views, leptos::html::q()),
                "rp" => html_element(&tag.clone(), child_views, leptos::html::rp()),
                "rt" => html_element(&tag.clone(), child_views, leptos::html::rt()),
                "ruby" => html_element(&tag.clone(), child_views, leptos::html::ruby()),
                "s" => html_element(&tag.clone(), child_views, leptos::html::s()),
                "samp" => html_element(&tag.clone(), child_views, leptos::html::samp()),
                "small" => html_element(&tag.clone(), child_views, leptos::html::small()),
                "span" => html_element(&tag.clone(), child_views, leptos::html::span()),
                "strong" => html_element(&tag.clone(), child_views, leptos::html::strong()),
                "sub" => html_element(&tag.clone(), child_views, leptos::html::sub()),
                "sup" => html_element(&tag.clone(), child_views, leptos::html::sup()),
                "time" => html_element(&tag.clone(), child_views, leptos::html::time()),
                "u" => html_element(&tag.clone(), child_views, leptos::html::u()),
                "var" => html_element(&tag.clone(), child_views, leptos::html::var()),
                "wbr" => html_element(&tag.clone(), child_views, leptos::html::wbr()),
                "area" => html_element(&tag.clone(), child_views, leptos::html::area()),
                "audio" => html_element(&tag.clone(), child_views, leptos::html::audio()),
                "img" => html_element(&tag.clone(), child_views, leptos::html::img()),
                "map" => html_element(&tag.clone(), child_views, leptos::html::map()),
                "track" => html_element(&tag.clone(), child_views, leptos::html::track()),
                "video" => html_element(&tag.clone(), child_views, leptos::html::video()),
                "embed" => html_element(&tag.clone(), child_views, leptos::html::embed()),
                "iframe" => html_element(&tag.clone(), child_views, leptos::html::iframe()),
                "object" => html_element(&tag.clone(), child_views, leptos::html::object()),
                "param" => html_element(&tag.clone(), child_views, leptos::html::param()),
                "picture" => html_element(&tag.clone(), child_views, leptos::html::picture()),
                "portal" => html_element(&tag.clone(), child_views, leptos::html::portal()),
                "source" => html_element(&tag.clone(), child_views, leptos::html::source()),
                "svg" => html_element(&tag.clone(), child_views, leptos::html::svg()),
                "math" => html_element(&tag.clone(), child_views, leptos::html::math()),
                "canvas" => html_element(&tag.clone(), child_views, leptos::html::canvas()),
                "noscript" => html_element(&tag.clone(), child_views, leptos::html::noscript()),
                "script" => html_element(&tag.clone(), child_views, leptos::html::script()),
                "del" => html_element(&tag.clone(), child_views, leptos::html::del()),
                "ins" => html_element(&tag.clone(), child_views, leptos::html::ins()),
                "caption" => html_element(&tag.clone(), child_views, leptos::html::caption()),
                "col" => html_element(&tag.clone(), child_views, leptos::html::col()),
                "colgroup" => html_element(&tag.clone(), child_views, leptos::html::colgroup()),
                "table" => html_element(&tag.clone(), child_views, leptos::html::table()),
                "tbody" => html_element(&tag.clone(), child_views, leptos::html::tbody()),
                "td" => html_element(&tag.clone(), child_views, leptos::html::td()),
                "tfoot" => html_element(&tag.clone(), child_views, leptos::html::tfoot()),
                "th" => html_element(&tag.clone(), child_views, leptos::html::th()),
                "thead" => html_element(&tag.clone(), child_views, leptos::html::thead()),
                "tr" => html_element(&tag.clone(), child_views, leptos::html::tr()),
                "button" => html_element(&tag.clone(), child_views, leptos::html::button()),
                "datalist" => html_element(&tag.clone(), child_views, leptos::html::datalist()),
                "fieldset" => html_element(&tag.clone(), child_views, leptos::html::fieldset()),
                "form" => html_element(&tag.clone(), child_views, leptos::html::form()),
                "input" => html_element(&tag.clone(), child_views, leptos::html::input()),
                "label" => html_element(&tag.clone(), child_views, leptos::html::label()),
                "legend" => html_element(&tag.clone(), child_views, leptos::html::legend()),
                "meter" => html_element(&tag.clone(), child_views, leptos::html::meter()),
                "optgroup" => html_element(&tag.clone(), child_views, leptos::html::optgroup()),
                "option" => html_element(&tag.clone(), child_views, leptos::html::option()),
                "output" => html_element(&tag.clone(), child_views, leptos::html::output()),
                "progress" => html_element(&tag.clone(), child_views, leptos::html::progress()),
                "select" => html_element(&tag.clone(), child_views, leptos::html::select()),
                "textarea" => html_element(&tag.clone(), child_views, leptos::html::textarea()),
                "details" => html_element(&tag.clone(), child_views, leptos::html::details()),
                "dialog" => html_element(&tag.clone(), child_views, leptos::html::dialog()),
                "menu" => html_element(&tag.clone(), child_views, leptos::html::menu()),
                "summary" => html_element(&tag.clone(), child_views, leptos::html::summary()),
                "slot" => html_element(&tag.clone(), child_views, leptos::html::slot()),
                "template" => html_element(&tag.clone(), child_views, leptos::html::template()),
                _ => {
                    println!("unknown element {}", name);
                    ().into_view()
                }
            }
        }
    }
}

fn html_element<Element>(
    element: &HTMLTag,
    children: Vec<View>,
    mut leptos_el: HtmlElement<Element>,
) -> View
where
    Element: ElementDescriptor + 'static,
{
    let attributes = element.attributes();

    let id = attributes.id().map(|id| id.as_utf8_str().to_string());

    let attributes_map: HashMap<String, Option<String>> = attributes
        .iter()
        .map(|(k, v)| (k.to_string(), v.map(|v| v.to_string())))
        .collect();

    let classes = attributes.class_iter().map_or(Vec::new(), |class_list| {
        class_list.map(|c| c.to_string()).collect()
    });

    if let Some(v) = id {
        leptos_el = leptos_el.id(v.clone());
    }

    for (k, v) in attributes_map {
        if let Some(v) = v {
            leptos_el = leptos_el.attr(k.clone(), v);
        }
    }

    if !classes.is_empty() {
        leptos_el = leptos_el.attr("class", classes.join(" "));
    }

    for child in children {
        leptos_el = leptos_el.child(child);
    }

    leptos_el.into_view()
}
