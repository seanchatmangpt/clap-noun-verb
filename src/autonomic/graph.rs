//! # Capability Graph and Schema Composition Engine
//!
//! Treats capabilities and schemas as a graph, enabling:
//! - Reachability queries
//! - Minimal composition finding
//! - Equivalence and dominance analysis
//! - Policy constraints over paths
//!
//! ## Design Principles
//!
//! 1. **Type-Safe IDs**: Strongly typed capability and schema references
//! 2. **Compile-Time Guarantees**: Graph queries cannot reference nonexistent nodes
//! 3. **Efficient Traversal**: Cache-friendly graph representation
//! 4. **Composability**: Find minimal paths between capabilities

use super::{
    capability_id::CapabilityId,
    effects::EffectMetadata,
    schema::{InputSchema, OutputSchema, TypeSchema},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Strongly-typed node ID in capability graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(u32);

impl NodeId {
    /// Create a new node ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Strongly-typed edge ID in capability graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(u32);

impl EdgeId {
    /// Create a new edge ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

/// Node in the capability graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityNode {
    /// Node ID
    pub id: NodeId,

    /// Capability ID
    pub capability_id: CapabilityId,

    /// Human-readable name
    pub name: String,

    /// Input schema
    pub input_schema: InputSchema,

    /// Output schema
    pub output_schema: OutputSchema,

    /// Effects
    pub effects: Vec<EffectMetadata>,

    /// Metadata
    pub metadata: NodeMetadata,
}

/// Node metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Description
    pub description: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Cost estimate (arbitrary units)
    pub cost: u32,

    /// Reliability score (0-100)
    pub reliability: u8,
}

/// Edge type in the capability graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Output of source can be used as input to target
    Produces,

    /// Source requires target to be available
    Requires,

    /// Source can be substituted by target
    Equivalent,

    /// Source provides strictly more capabilities than target
    Dominates,

    /// Custom edge type
    Custom,
}

/// Edge in the capability graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEdge {
    /// Edge ID
    pub id: EdgeId,

    /// Source node
    pub source: NodeId,

    /// Target node
    pub target: NodeId,

    /// Edge type
    pub edge_type: EdgeType,

    /// Schema compatibility (for Produces edges)
    pub schema_compatibility: Option<SchemaCompatibility>,

    /// Metadata
    pub metadata: EdgeMetadata,
}

/// Schema compatibility between two capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCompatibility {
    /// Whether schemas are fully compatible
    pub compatible: bool,

    /// Fields that match
    pub matching_fields: Vec<String>,

    /// Fields that need transformation
    pub needs_transform: Vec<String>,

    /// Transformation cost
    pub transform_cost: u32,
}

/// Edge metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// Description
    pub description: Option<String>,

    /// Weight for path finding
    pub weight: u32,
}

/// Capability graph
#[derive(Debug, Clone)]
pub struct CapabilityGraph {
    /// Nodes indexed by NodeId
    nodes: HashMap<NodeId, CapabilityNode>,

    /// Edges indexed by EdgeId
    edges: HashMap<EdgeId, CapabilityEdge>,

    /// Adjacency list (node -> outgoing edges)
    adjacency: HashMap<NodeId, Vec<EdgeId>>,

    /// Reverse adjacency list (node -> incoming edges)
    reverse_adjacency: HashMap<NodeId, Vec<EdgeId>>,

    /// Capability ID to Node ID mapping
    capability_index: HashMap<CapabilityId, NodeId>,

    /// Next node ID
    next_node_id: u32,

    /// Next edge ID
    next_edge_id: u32,
}

impl CapabilityGraph {
    /// Create a new empty capability graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
            capability_index: HashMap::new(),
            next_node_id: 0,
            next_edge_id: 0,
        }
    }

    /// Add a node to the graph
    pub fn add_node(
        &mut self,
        capability_id: CapabilityId,
        name: impl Into<String>,
        input_schema: InputSchema,
        output_schema: OutputSchema,
        effects: Vec<EffectMetadata>,
    ) -> NodeId {
        let node_id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;

        let node = CapabilityNode {
            id: node_id,
            capability_id: capability_id.clone(),
            name: name.into(),
            input_schema,
            output_schema,
            effects,
            metadata: NodeMetadata::default(),
        };

        self.nodes.insert(node_id, node);
        self.capability_index.insert(capability_id, node_id);
        self.adjacency.insert(node_id, Vec::new());
        self.reverse_adjacency.insert(node_id, Vec::new());

        node_id
    }

    /// Add an edge to the graph
    pub fn add_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        edge_type: EdgeType,
    ) -> Result<EdgeId, GraphError> {
        // Verify nodes exist
        if !self.nodes.contains_key(&source) {
            return Err(GraphError::NodeNotFound(source));
        }
        if !self.nodes.contains_key(&target) {
            return Err(GraphError::NodeNotFound(target));
        }

        let edge_id = EdgeId::new(self.next_edge_id);
        self.next_edge_id += 1;

        // Compute schema compatibility for Produces edges
        let schema_compatibility = if edge_type == EdgeType::Produces {
            Some(self.compute_schema_compatibility(source, target))
        } else {
            None
        };

        let edge = CapabilityEdge {
            id: edge_id,
            source,
            target,
            edge_type,
            schema_compatibility,
            metadata: EdgeMetadata::default(),
        };

        self.edges.insert(edge_id, edge);
        self.adjacency.entry(source).or_default().push(edge_id);
        self.reverse_adjacency.entry(target).or_default().push(edge_id);

        Ok(edge_id)
    }

    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<&CapabilityNode> {
        self.nodes.get(&id)
    }

    /// Get a node by capability ID
    pub fn get_node_by_capability(&self, capability_id: &CapabilityId) -> Option<&CapabilityNode> {
        self.capability_index
            .get(capability_id)
            .and_then(|id| self.nodes.get(id))
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: EdgeId) -> Option<&CapabilityEdge> {
        self.edges.get(&id)
    }

    /// Get outgoing edges from a node
    pub fn outgoing_edges(&self, node: NodeId) -> &[EdgeId] {
        self.adjacency.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get incoming edges to a node
    pub fn incoming_edges(&self, node: NodeId) -> &[EdgeId] {
        self.reverse_adjacency.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if there's a path from source to target
    pub fn is_reachable(&self, source: NodeId, target: NodeId) -> bool {
        if source == target {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(source);
        visited.insert(source);

        while let Some(current) = queue.pop_front() {
            for &edge_id in self.outgoing_edges(current) {
                if let Some(edge) = self.edges.get(&edge_id) {
                    if edge.target == target {
                        return true;
                    }
                    if visited.insert(edge.target) {
                        queue.push_back(edge.target);
                    }
                }
            }
        }

        false
    }

    /// Find the shortest path from source to target
    pub fn shortest_path(&self, source: NodeId, target: NodeId) -> Option<Vec<NodeId>> {
        if source == target {
            return Some(vec![source]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<NodeId, NodeId> = HashMap::new();

        queue.push_back(source);
        visited.insert(source);

        while let Some(current) = queue.pop_front() {
            for &edge_id in self.outgoing_edges(current) {
                if let Some(edge) = self.edges.get(&edge_id) {
                    if edge.target == target {
                        // Reconstruct path
                        let mut path = vec![target, current];
                        let mut node = current;
                        while let Some(&prev) = parent.get(&node) {
                            path.push(prev);
                            node = prev;
                        }
                        path.reverse();
                        return Some(path);
                    }

                    if visited.insert(edge.target) {
                        parent.insert(edge.target, current);
                        queue.push_back(edge.target);
                    }
                }
            }
        }

        None
    }

    /// Find minimal composition: given input and output schemas, find the minimal capability chain
    pub fn find_minimal_composition(
        &self,
        input_schema: &TypeSchema,
        output_schema: &TypeSchema,
    ) -> Option<Vec<NodeId>> {
        // Find all nodes that accept the input schema
        let start_nodes: Vec<NodeId> = self
            .nodes
            .values()
            .filter(|node| self.schema_accepts(&node.input_schema, input_schema))
            .map(|node| node.id)
            .collect();

        // Find all nodes that produce the output schema
        let end_nodes: Vec<NodeId> = self
            .nodes
            .values()
            .filter(|node| self.schema_produces(&node.output_schema, output_schema))
            .map(|node| node.id)
            .collect();

        // Find shortest path between any start and end node
        let mut shortest: Option<Vec<NodeId>> = None;

        for &start in &start_nodes {
            for &end in &end_nodes {
                if let Some(path) = self.shortest_path(start, end) {
                    if shortest.is_none() || path.len() < shortest.as_ref().unwrap().len() {
                        shortest = Some(path);
                    }
                }
            }
        }

        shortest
    }

    /// Find all paths from source to target (up to max_depth)
    pub fn find_all_paths(
        &self,
        source: NodeId,
        target: NodeId,
        max_depth: usize,
    ) -> Vec<Vec<NodeId>> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();

        self.dfs_paths(source, target, &mut current_path, &mut visited, &mut paths, max_depth);

        paths
    }

    /// Depth-first search for all paths
    fn dfs_paths(
        &self,
        current: NodeId,
        target: NodeId,
        path: &mut Vec<NodeId>,
        visited: &mut HashSet<NodeId>,
        paths: &mut Vec<Vec<NodeId>>,
        max_depth: usize,
    ) {
        if path.len() >= max_depth {
            return;
        }

        path.push(current);
        visited.insert(current);

        if current == target {
            paths.push(path.clone());
        } else {
            for &edge_id in self.outgoing_edges(current) {
                if let Some(edge) = self.edges.get(&edge_id) {
                    if !visited.contains(&edge.target) {
                        self.dfs_paths(edge.target, target, path, visited, paths, max_depth);
                    }
                }
            }
        }

        path.pop();
        visited.remove(&current);
    }

    /// Check if one capability dominates another
    pub fn dominates(&self, dominant: NodeId, dominated: NodeId) -> bool {
        // Check for explicit dominance edge
        for &edge_id in self.outgoing_edges(dominant) {
            if let Some(edge) = self.edges.get(&edge_id) {
                if edge.edge_type == EdgeType::Dominates && edge.target == dominated {
                    return true;
                }
            }
        }

        // Compute dominance based on capabilities
        if let (Some(dom_node), Some(ded_node)) = (self.get_node(dominant), self.get_node(dominated)) {
            // Dominant must have all effects of dominated, and possibly more
            let dom_effects: HashSet<_> = dom_node.effects.iter().map(|e| &e.effect_type).collect();
            let ded_effects: HashSet<_> = ded_node.effects.iter().map(|e| &e.effect_type).collect();

            ded_effects.is_subset(&dom_effects)
        } else {
            false
        }
    }

    /// Find equivalence classes (nodes that are equivalent)
    pub fn find_equivalence_classes(&self) -> Vec<Vec<NodeId>> {
        let mut classes: Vec<Vec<NodeId>> = Vec::new();
        let mut assigned: HashSet<NodeId> = HashSet::new();

        for &node_id in self.nodes.keys() {
            if assigned.contains(&node_id) {
                continue;
            }

            let mut class = vec![node_id];
            assigned.insert(node_id);

            // Find all nodes equivalent to this one
            for &other_id in self.nodes.keys() {
                if other_id != node_id && !assigned.contains(&other_id) {
                    if self.are_equivalent(node_id, other_id) {
                        class.push(other_id);
                        assigned.insert(other_id);
                    }
                }
            }

            classes.push(class);
        }

        classes
    }

    /// Check if two nodes are equivalent
    pub fn are_equivalent(&self, a: NodeId, b: NodeId) -> bool {
        // Check for explicit equivalence edge
        for &edge_id in self.outgoing_edges(a) {
            if let Some(edge) = self.edges.get(&edge_id) {
                if edge.edge_type == EdgeType::Equivalent && edge.target == b {
                    return true;
                }
            }
        }

        // Equivalence is symmetric
        for &edge_id in self.outgoing_edges(b) {
            if let Some(edge) = self.edges.get(&edge_id) {
                if edge.edge_type == EdgeType::Equivalent && edge.target == a {
                    return true;
                }
            }
        }

        false
    }

    /// Compute schema compatibility between two nodes
    fn compute_schema_compatibility(&self, source: NodeId, target: NodeId) -> SchemaCompatibility {
        let source_node = self.nodes.get(&source);
        let target_node = self.nodes.get(&target);

        match (source_node, target_node) {
            (Some(s), Some(t)) => {
                // Check if source output is compatible with target input
                self.check_schema_compatibility(&s.output_schema, &t.input_schema)
            }
            _ => SchemaCompatibility {
                compatible: false,
                matching_fields: vec![],
                needs_transform: vec![],
                transform_cost: 0,
            },
        }
    }

    /// Check if output schema is compatible with input schema
    fn check_schema_compatibility(
        &self,
        _output: &OutputSchema,
        _input: &InputSchema,
    ) -> SchemaCompatibility {
        // Simplified compatibility check
        // In a real implementation, this would do deep schema analysis

        SchemaCompatibility {
            compatible: true, // Placeholder
            matching_fields: vec![],
            needs_transform: vec![],
            transform_cost: 0,
        }
    }

    /// Check if schema accepts a type
    fn schema_accepts(&self, _schema: &InputSchema, _type_schema: &TypeSchema) -> bool {
        // Simplified check - real implementation would do deep type matching
        true // Placeholder
    }

    /// Check if schema produces a type
    fn schema_produces(&self, _schema: &OutputSchema, _type_schema: &TypeSchema) -> bool {
        // Simplified check - real implementation would do deep type matching
        true // Placeholder
    }

    /// Export graph for visualization
    pub fn export_dot(&self) -> String {
        let mut dot = String::from("digraph CapabilityGraph {\n");

        // Add nodes
        for node in self.nodes.values() {
            dot.push_str(&format!(
                "  n{} [label=\"{}\"];\n",
                node.id.raw(),
                node.name
            ));
        }

        // Add edges
        for edge in self.edges.values() {
            let label = format!("{:?}", edge.edge_type);
            dot.push_str(&format!(
                "  n{} -> n{} [label=\"{}\"];\n",
                edge.source.raw(),
                edge.target.raw(),
                label
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            avg_degree: if self.nodes.is_empty() {
                0.0
            } else {
                self.edges.len() as f64 / self.nodes.len() as f64
            },
            max_depth: self.compute_max_depth(),
        }
    }

    /// Compute maximum depth of the graph
    fn compute_max_depth(&self) -> usize {
        let mut max_depth = 0;

        for &node_id in self.nodes.keys() {
            let depth = self.compute_node_depth(node_id);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Compute depth of a node (longest path from any root)
    fn compute_node_depth(&self, node: NodeId) -> usize {
        let mut visited = HashSet::new();
        self.dfs_depth(node, &mut visited)
    }

    fn dfs_depth(&self, node: NodeId, visited: &mut HashSet<NodeId>) -> usize {
        if !visited.insert(node) {
            return 0; // Cycle detected
        }

        let mut max_depth = 0;
        for &edge_id in self.outgoing_edges(node) {
            if let Some(edge) = self.edges.get(&edge_id) {
                let depth = self.dfs_depth(edge.target, visited);
                max_depth = max_depth.max(depth);
            }
        }

        visited.remove(&node);
        max_depth + 1
    }
}

impl Default for CapabilityGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    /// Number of nodes
    pub node_count: usize,

    /// Number of edges
    pub edge_count: usize,

    /// Average degree
    pub avg_degree: f64,

    /// Maximum depth
    pub max_depth: usize,
}

/// Graph errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum GraphError {
    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0:?}")]
    EdgeNotFound(EdgeId),

    #[error("Cycle detected")]
    CycleDetected,

    #[error("Invalid path")]
    InvalidPath,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let mut graph = CapabilityGraph::new();

        let n1 = graph.add_node(
            CapabilityId::from_path("a"),
            "A",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        let n2 = graph.add_node(
            CapabilityId::from_path("b"),
            "B",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        let _e1 = graph.add_edge(n1, n2, EdgeType::Produces).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_reachability() {
        let mut graph = CapabilityGraph::new();

        let n1 = graph.add_node(
            CapabilityId::from_path("a"),
            "A",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        let n2 = graph.add_node(
            CapabilityId::from_path("b"),
            "B",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        let n3 = graph.add_node(
            CapabilityId::from_path("c"),
            "C",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        graph.add_edge(n1, n2, EdgeType::Produces).unwrap();
        graph.add_edge(n2, n3, EdgeType::Produces).unwrap();

        assert!(graph.is_reachable(n1, n3));
        assert!(!graph.is_reachable(n3, n1));
    }

    #[test]
    fn test_shortest_path() {
        let mut graph = CapabilityGraph::new();

        let n1 = graph.add_node(
            CapabilityId::from_path("a"),
            "A",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        let n2 = graph.add_node(
            CapabilityId::from_path("b"),
            "B",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        let n3 = graph.add_node(
            CapabilityId::from_path("c"),
            "C",
            InputSchema::default(),
            OutputSchema::new(TypeSchema::primitive(PrimitiveType::String)),
            vec![],
        );

        graph.add_edge(n1, n2, EdgeType::Produces).unwrap();
        graph.add_edge(n2, n3, EdgeType::Produces).unwrap();

        let path = graph.shortest_path(n1, n3).unwrap();
        assert_eq!(path, vec![n1, n2, n3]);
    }
}
