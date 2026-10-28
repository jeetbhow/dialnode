//! JSON export in the format documented in the README.

use serde_json::{Value, json};

use crate::model::{Graph, NodeData};

pub fn export_graph(graph: &Graph) -> Value {
    let nodes: Vec<Value> = graph
        .nodes
        .iter()
        .map(|node| {
            let id = &node.id;
            let kind = node.node_type().as_str();
            match &node.data {
                NodeData::Start(d) => json!({ "id": id, "type": kind, "next": d.next }),
                NodeData::Text(d) => {
                    let mut v = json!({ "id": id, "type": kind, "text": d.text });
                    if let Some(speaker) = &d.speaker {
                        v["speaker"] = json!(speaker.name);
                    }
                    if let Some(portrait) = &d.portrait {
                        v["portrait"] = json!(portrait.virtual_path);
                    }
                    v["next"] = json!(d.next);
                    v
                }
                NodeData::BranchContainer(d) => {
                    json!({ "id": id, "type": kind, "branches": d.branches })
                }
                NodeData::Branch(d) => json!({
                    "id": id,
                    "parentId": node.parent_id,
                    "type": kind,
                    "text": d.text,
                    "next": d.next,
                }),
                NodeData::SkillCheck(d) => json!({
                    "id": id,
                    "parentId": node.parent_id,
                    "type": kind,
                    "text": d.text,
                    "skill": d.skill.as_ref().map(|s| &s.name),
                    "difficulty": d.difficulty,
                    "next": d.next,
                }),
                NodeData::End => json!({ "id": id, "type": kind }),
            }
        })
        .collect();
    Value::Array(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeType, Speaker};
    use eframe::egui::pos2;

    #[test]
    fn matches_documented_format() {
        let mut g = Graph::default();
        let start = g.add_node(NodeType::Start, pos2(0.0, 0.0));
        let text = g.add_node(NodeType::Text, pos2(0.0, 0.0));
        g.connect(&start, None, &text, None);
        if let NodeData::Text(d) = &mut g.node_mut(&text).unwrap().data {
            d.text = "What's your favourite color?".into();
            d.speaker = Some(Speaker::new("Jeet".into()));
        }

        let out = serde_json::to_string(&export_graph(&g)).unwrap();
        let expected = format!(
            r#"[{{"id":"{start}","type":"start","next":"{text}"}},{{"id":"{text}","type":"text","text":"What's your favourite color?","speaker":"Jeet","next":null}}]"#
        );
        assert_eq!(out, expected);
    }
}
