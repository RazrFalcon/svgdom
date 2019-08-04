use std::str;

use crate::{
    AttributeId,
    AttributeValue,
    Document,
    Node,
};

trait StrTrim {
    fn remove_first(&mut self);
    fn remove_last(&mut self);
}

impl StrTrim for String {
    fn remove_first(&mut self) {
        self.drain(0..1);
    }

    fn remove_last(&mut self) {
        self.pop();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum XmlSpace {
    Default,
    Preserve,
}

// Prepare text nodes according to the spec: https://www.w3.org/TR/SVG11/text.html#WhiteSpace
//
// This function handles:
// - 'xml:space' processing
// - tabs and newlines removing/replacing
// - spaces trimming
pub fn prepare_text(doc: &mut Document) {
    // Remember nodes that have 'xml:space' changed.
    let mut nodes = Vec::new();

    _prepare_text(&doc.root(), &mut nodes, XmlSpace::Default);

    // Remove temporary 'xml:space' attributes created during the text processing.
    for mut node in nodes {
        node.remove_attribute(AttributeId::Space);
    }

    let root = doc.root().clone();
    doc.drain(root, |n| n.is_text() && n.text().is_empty());
}

fn _prepare_text(parent: &Node, nodes: &mut Vec<Node>, parent_xmlspace: XmlSpace) {
    for mut node in parent.children().filter(|n| n.is_element()) {
        let xmlspace = get_xmlspace(&mut node, nodes, parent_xmlspace);

        if let Some(child) = node.first_child() {
            if child.is_text() {
                prepare_text_children(&node, nodes, xmlspace);
                continue;
            }
        }

        _prepare_text(&node, nodes, xmlspace);
    }
}

fn get_xmlspace(node: &mut Node, nodes: &mut Vec<Node>, default: XmlSpace) -> XmlSpace {
    {
        let attrs = node.attributes();
        let v = attrs.get_value(AttributeId::Space);
        if let Some(&AttributeValue::String(ref s)) = v {
            return if s == "preserve" { XmlSpace::Preserve } else { XmlSpace::Default };
        }
    }

    // 'xml:space' is not set - set it manually.
    set_xmlspace(node, nodes, default);

    default
}

fn set_xmlspace(node: &mut Node, nodes: &mut Vec<Node>, xmlspace: XmlSpace) {
    let xmlspace_str = match xmlspace {
        XmlSpace::Default => "default",
        XmlSpace::Preserve => "preserve",
    };

    node.set_attribute((AttributeId::Space, xmlspace_str));

    nodes.push(node.clone());
}

fn prepare_text_children(parent: &Node, marked_nodes: &mut Vec<Node>, xmlspace: XmlSpace) {
    // Trim all descendant text nodes.
    for mut child in parent.descendants() {
        if child.is_text() {
            let child_xmlspace = get_xmlspace(&mut child.parent().unwrap(), marked_nodes, xmlspace);
            let new_text = {
                let text = child.text();
                trim(text.as_ref(), child_xmlspace)
            };
            child.set_text(&new_text);
        }
    }

    let mut nodes = Vec::new();
    collect_text(parent, 0, &mut nodes);

    // `trim` method has already collapsed all spaces into a single one,
    // so we have to check only for one leading or trailing space.

    if nodes.len() == 1 {
        // Process element with a single text node child.

        let mut node = nodes[0].0.clone();

        if xmlspace == XmlSpace::Default {
            let mut text = node.text_mut();

            match text.len() {
                0 => {} // An empty string. Do nothing.
                1 => {
                    // If string has only one character and it's a space - clear this string.
                    if text.as_bytes()[0] == b' ' {
                        text.clear();
                    }
                }
                _ => {
                    // 'text' has at least 2 bytes, so indexing is safe.
                    let c1 = text.as_bytes()[0];
                    let c2 = text.as_bytes()[text.len() - 1];

                    if c1 == b' ' {
                        text.remove_first();
                    }

                    if c2 == b' ' {
                        text.remove_last();
                    }
                }
            }
        } else {
            // Do nothing when xml:space=preserve.
        }
    } else if nodes.len() > 1 {
        // Process element with many text node children.

        // We manage all text nodes as a single text node
        // and trying to remove duplicated spaces across nodes.
        //
        // For example    '<text>Text <tspan> text </tspan> text</text>'
        // is the same is '<text>Text <tspan>text</tspan> text</text>'

        let mut i = 0;
        let len = nodes.len() - 1;
        let mut last_non_empty: Option<Node> = None;
        while i < len {
            // Process pairs.
            let (mut node1, depth1) = nodes[i].clone();
            let (mut node2, depth2) = nodes[i + 1].clone();

            if node1.text().is_empty() {
                if let Some(ref n) = last_non_empty {
                    node1 = n.clone();
                }
            }

            // Parent of the text node is always an element node and always exist,
            // so unwrap is safe.
            let xmlspace1 = get_xmlspace(&mut node1.parent().unwrap(), marked_nodes, xmlspace);
            let xmlspace2 = get_xmlspace(&mut node2.parent().unwrap(), marked_nodes, xmlspace);

            // >text<..>text<
            //  1  2    3  4
            let (c1, c2, c3, c4) = {
                let text1 = node1.text();
                let text2 = node2.text();

                let bytes1 = text1.as_bytes();
                let bytes2 = text2.as_bytes();

                let c1 = bytes1.first().cloned();
                let c2 = bytes1.last().cloned();
                let c3 = bytes2.first().cloned();
                let c4 = bytes2.last().cloned();

                (c1, c2, c3, c4)
            };

            // NOTE: xml:space processing is mostly an undefined behavior,
            // because everyone do this differently.
            // We mimic Chrome behavior.

            // Remove space from the second text node if both nodes has bound spaces.
            // From: '<text>Text <tspan> text</tspan></text>'
            // To:   '<text>Text <tspan>text</tspan></text>'
            //
            // See text-tspan-02-b.svg for details.
            if c2 == Some(b' ') && c2 == c3 {
                if depth1 < depth2 {
                    if xmlspace2 == XmlSpace::Default {
                        node2.text_mut().remove_first();
                    }
                } else {
                    if xmlspace1 == XmlSpace::Default && xmlspace2 == XmlSpace::Default {
                        node1.text_mut().remove_last();
                    } else if xmlspace1 == XmlSpace::Preserve && xmlspace2 == XmlSpace::Default {
                        node2.text_mut().remove_first();
                    }
                }
            }

            let is_first = i == 0;
            let is_last  = i == len - 1;

            if     is_first
                && c1 == Some(b' ')
                && xmlspace1 == XmlSpace::Default
                && !node1.text().is_empty()
            {
                // Remove leading space of the first text node.
                node1.text_mut().remove_first();
            } else if    is_last
                      && c4 == Some(b' ')
                      && !node2.text().is_empty()
                      && xmlspace2 == XmlSpace::Default
            {
                // Remove trailing space of the last text node.
                // Also check that 'text2' is not empty already.
                node2.text_mut().remove_last();
            }

            if     is_last
                && c2 == Some(b' ')
                && !node1.text().is_empty()
                && node2.text().is_empty()
                && node1.text().ends_with(' ')
            {
                node1.text_mut().remove_last();
            }

            if !node1.text().trim().is_empty() {
                last_non_empty = Some(node1.clone());
            }

            i += 1;
        }
    }
}

fn collect_text(parent: &Node, depth: usize, nodes: &mut Vec<(Node, usize)>) {
    for child in parent.children() {
        if child.is_text() {
            nodes.push((child.clone(), depth));
        } else if child.is_element() {
            collect_text(&child, depth + 1, nodes);
        }
    }
}

fn trim(text: &str, space: XmlSpace) -> String {
    let mut s = String::with_capacity(text.len());

    let mut prev = '0';
    for c in text.chars() {
        // \r, \n and \t should be converted into spaces.
        let c = match c {
            '\r' | '\n' | '\t' => ' ',
            _ => c,
        };

        // Skip continuous spaces.
        if space == XmlSpace::Default && c == ' ' && c == prev {
            continue;
        }

        prev = c;

        s.push(c);
    }

    s
}
