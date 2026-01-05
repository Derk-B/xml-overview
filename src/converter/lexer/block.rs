use std::collections::HashMap;

use crate::converter::lexer::token::Token;

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    size: usize,
    current: usize,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            nodes: Vec::from([Node {
                id: 0,
                name: String::from("0"),
                keys: Vec::new(),
                parent: None,
                children: Vec::new(),
            }]),
            size: 1,
            current: 0,
        }
    }

    pub fn add_node(&mut self, name: &String, keys: &Vec<String>) {
        let new_node_id = self.size;
        self.nodes.push(Node {
            id: new_node_id,
            name: name.clone(),
            keys: keys.clone(),
            parent: Some(self.current),
            children: Vec::new(),
        });

        let parent = &mut self.nodes[self.current];
        parent.children.push(NodeOrToken::N(new_node_id));

        self.size += 1;
        self.current = new_node_id;
    }

    pub fn add_token(&mut self, token: &Token) {
        let current_node = &mut self.nodes[self.current];
        current_node.children.push(NodeOrToken::T(token.clone()));
    }

    pub fn get_node(&self, idx: usize) -> Option<&Node> {
        self.nodes.get(idx)
    }

    pub fn print(&self) -> String {
        let mut res = String::from("");
        res += &self.nodes[0].print(&self);

        res
    }

    pub fn close_current(&mut self) {
        if let Some(idx) = self.nodes[self.current].parent {
            self.current = idx;
        }
    }

    /// Minimizes the XML graph by removing nodes with the same keys, keeps the node with the most children.
    pub fn minimize(&mut self) {
        let mut updated_nodes = self.nodes.clone();
        for child_index in &self.nodes[self.current]
            .children
            .iter()
            .filter(|n| match n {
                NodeOrToken::N(_) => true,
                _ => false,
            })
            .map(|n| match n {
                NodeOrToken::N(i) => i.clone(),
                _ => panic!("Fatal error: Filter failed somehow"),
            })
            .collect::<Vec<usize>>()
        {
            let nodes = &self.nodes.clone();
            let new_nodes = self.nodes[*child_index].minimize(nodes);
            for (i, n) in new_nodes.iter().enumerate() {
                if updated_nodes[i].children.len() > n.children.len() {
                    updated_nodes[i] = n.clone();
                }
            }
        }

        self.nodes = updated_nodes;
    }

    pub fn print_tree(&self) {
        for node in &self.nodes {
            node.print_tree(self);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub keys: Vec<String>,
    pub parent: Option<usize>,
    pub children: Vec<NodeOrToken>,
}

impl Node {
    pub fn print(&self, graph: &Graph) -> String {
        let mut res = String::from("");

        res += &format!("<{}", self.name);

        if self.keys.len() > 0 {
            res += " ";
        }

        res += &self
            .keys
            .iter()
            .map(|k| format!("{}=\"\"", k))
            .collect::<Vec<String>>()
            .join(" ");

        if self.children.len() == 0 {
            res += "/>";
            return res;
        } else {
            res += ">";
        }

        for child in &self.children {
            match child {
                NodeOrToken::N(ni) => {
                    if let Some(node) = graph.get_node(*ni) {
                        res += &node.print(graph);
                    }
                }
                NodeOrToken::T(t) => match t {
                    Token::Whitespace => res += " ",
                    Token::Newline => {
                        if !res.ends_with("\n") {
                            res += "\n";
                        }
                    }
                    Token::Comment(comment) => (), // res += &format!("<!-- {} -->", comment),
                    Token::Text(txt) => res += txt,
                    _ => (),
                },
            }
        }

        res += &format!("</{}>", self.name);

        res
    }

    fn minimize(&mut self, nodes: &Vec<Node>) -> Vec<Node> {
        let mut child_nodes = Vec::new();
        for child_index in &self.children {
            if let NodeOrToken::N(n) = child_index {
                child_nodes.push(nodes[*n].clone());
            }
        }

        let mut child_map = HashMap::<String, &Node>::new();
        for child in &child_nodes {
            let unique_child_key = child.name.clone() + "," + &child.keys.join(",");
            if let Some(c) = child_map.get(&unique_child_key) {
                if c.count_child_nodes() < child.count_child_nodes() {
                    child_map.insert(unique_child_key, child);
                }
            } else {
                child_map.insert(unique_child_key, child);
            }
        }

        let remaining_child_ids = child_map.values().map(|c| c.id).collect::<Vec<usize>>();

        self.children = self
            .children
            .iter()
            .filter(|c| match c {
                NodeOrToken::T(_) => true,
                NodeOrToken::N(id) => remaining_child_ids.contains(id),
            })
            .cloned()
            .collect::<Vec<NodeOrToken>>();

        let mut updated_nodes = nodes.clone();
        updated_nodes[self.id].children = self.children.clone();
        for node_or_token in self.children.clone() {
            if let NodeOrToken::N(i) = node_or_token {
                let new_nodes = nodes[i].clone().minimize(nodes);
                for (ni, new_node) in new_nodes.iter().enumerate() {
                    if updated_nodes[ni].children.len() > new_node.children.len() {
                        updated_nodes[ni] = new_node.clone();
                    }
                }
            }
        }

        updated_nodes
    }

    fn count_child_nodes(&self) -> usize {
        self.children
            .iter()
            .filter(|n| match n {
                NodeOrToken::N(_) => true,
                _ => false,
            })
            .count()
    }

    fn print_tree(&self, graph: &Graph) {
        println!(
            "{}, children: [{:?}]",
            self.name,
            self.children
                .iter()
                .filter(|n| match n {
                    NodeOrToken::N(_) => true,
                    _ => false,
                })
                .map(|n| match n {
                    NodeOrToken::N(i) => graph.nodes[*i].name.clone(),
                    _ => "".to_string(),
                })
                .collect::<Vec<String>>()
                .join(",")
        );

        for child in &self.children {
            if let NodeOrToken::N(n) = child {
                graph.nodes[*n].print_tree(graph);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeOrToken {
    T(Token),
    N(usize),
}
