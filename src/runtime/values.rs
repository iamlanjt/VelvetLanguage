#[derive(Debug, Clone)]
pub enum RuntimeVal {
    NumberVal(NumberVal),
    NullVal(NullVal)
}

#[derive(Debug, Clone)]
pub struct NumberVal {
    pub value: usize
}

#[derive(Debug, Clone)]
pub struct NullVal {

}