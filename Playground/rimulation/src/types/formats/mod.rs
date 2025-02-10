pub mod custom;

pub trait NamedComponent {
    fn get_name(&self) -> String;
}
