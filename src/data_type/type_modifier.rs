#[derive(Debug, Clone, PartialEq)]
pub enum DeclModifier {
    POINTER,//this declaration is for a pointer to something
    ARRAY(u64),//an array with usize elements
    UnknownSizeArray,//int x[] = {1,2,3};
}