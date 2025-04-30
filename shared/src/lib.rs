#![allow(dead_code)]
pub mod data;
pub mod sqlite;
pub mod crypto;
pub mod filesystem;
pub mod net;

// #[macro_export]
// macro_rules! fpos {
//    () => {{
//       fn f() {}
//       fn type_name_of<T>(_: T) -> &'static str {
//          std::any::type_name::<T>()
//       }
//       let fname = type_name_of(f);
//       let name = match &fname[..fname.len()-3].rfind(':') {
//             Some(idx) => &fname[idx+1..fname.len()-3],
//             _ => &fname[..fname.len()-3]
//          };
//       &format!("[{}::{}:{}] Error", file!(), name, line!())[..]
//    }}
// }

fn type_of<T>(_: &T) -> String {
    std::any::type_name::<T>().to_string()
}

