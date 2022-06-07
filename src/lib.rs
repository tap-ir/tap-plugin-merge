//! Merge plugin concatenate dynamically different VFile into one VFile
//! there is no allocation or copy of the content the underlying content is read and copyied
//! only when the merged file is accessed

use std::sync::Arc;

use tap::plugin;
use tap::node::Node;
use tap::config_schema;
use tap::vfile::VFileBuilder;
use tap::error::RustructError;
use tap::mappedvfile::{MappedVFileBuilder,FileRanges};
use tap::tree::{TreeNodeId, TreeNodeIdSchema, VecTreeNodeIdSchema};
use tap::plugin::{PluginInfo, PluginInstance, PluginConfig, PluginArgument, PluginResult, PluginEnvironment};

use schemars::{JsonSchema};
use serde::{Serialize, Deserialize};

plugin!("merge", "Util", "Merge files into one file", Merge, Arguments);


#[derive(Default)]
pub struct Merge 
{
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Arguments
{
  name  : String,
  #[schemars(with = "VecTreeNodeIdSchema")] 
  files : Vec<TreeNodeId>,
  #[schemars(with = "TreeNodeIdSchema")] 
  mount_point : TreeNodeId,
}

#[derive(Debug, Serialize, Deserialize,Default)]
pub struct Results
{
}

impl Merge 
{
    pub fn merge(&self, builders : Vec<Arc<dyn VFileBuilder>>) -> Arc<dyn VFileBuilder>
    {
      let mut offset = 0;
      let mut file_ranges = FileRanges::new();

      for builder in builders
      {

        let range = offset .. offset + builder.size();
        offset += builder.size();

        file_ranges.push(range, 0, builder);
      }
      Arc::new(MappedVFileBuilder::new(file_ranges))
    }

    fn run(&mut self, args : Arguments, env : PluginEnvironment) -> anyhow::Result<Results>
    {
      let mut builders : Vec<Arc<dyn VFileBuilder>> = Vec::new(); 

      for file in args.files.into_iter()
      {
        let file_node = env.tree.get_node_from_id(file).ok_or(RustructError::ArgumentNotFound("file"))?;
        file_node.value().add_attribute(self.name(), None, None); 
        let data = file_node.value().get_value("data").ok_or(RustructError::ValueNotFound("data"))?;
        let data_builder = data.try_as_vfile_builder().ok_or(RustructError::ValueTypeMismatch)?;
        builders.push(data_builder);
      }

      let vfile_builder = self.merge(builders);

      let merged_file_node = Node::new(args.name);
      merged_file_node.value().add_attribute("data", vfile_builder, None);
      env.tree.add_child(args.mount_point, merged_file_node).unwrap();

      Ok(Results{})
    }
}
