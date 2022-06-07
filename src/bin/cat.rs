//! Cat is a binary that take a list of file in inputs, concatenate them and output them to the standard output

extern crate tap_plugin_merge;
extern crate tap_plugin_local;

use std::env;
use std::sync::Arc;

use tap::vfile::VFileBuilder;
use tap_plugin_local::LocalVFileBuilder;
use tap_plugin_merge::Merge;

fn main() 
{
   if env::args().len() < 2
   {
     println!("cat input_files");
     return ;
   }

   let files_path : Vec<String> = env::args().skip(1).collect();
   let mut builders : Vec<Arc< dyn VFileBuilder> > = Vec::new();

   for file_path in files_path
   {
      let vfile_builder = match LocalVFileBuilder::new(file_path) 
      {
        Ok(vfile_builder) => Arc::new(vfile_builder),
        Err(err) => {
                      println!("{}", err);
                      return 
                    },
      };
      builders.push(vfile_builder);
   }

   let merge = Merge{};
   let merged = merge.merge(builders);

   let mut file = merged.open().unwrap();
   let mut buffer = String::new();
   file.read_to_string(&mut buffer).unwrap();
   println!("{}", buffer); 
}
