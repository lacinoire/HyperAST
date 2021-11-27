#[derive(Debug)]
pub struct FullNode<Global,Local> {
    pub(crate) global: Global,    
    pub(crate) local: Local, 
}

// pub struct FullNode {
//     compressible_node: NodeIdentifier,
//     depth: usize,
//     position: usize,
//     height: u32,
//     size: u32,
//     hashs: SyntaxNodeHashs<u32>,
// }

// impl FullNode {
//     pub fn id(&self) -> &NodeIdentifier {
//         &self.compressible_node
//     }
// }