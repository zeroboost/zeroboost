use anchor_lang::Result;

pub trait Validate {
    fn validate(&self) -> Result<()>;
}
