extern crate syn;


use syn::Body;

struct FieldsHelper<'a> {
    body: &'a Body
}

impl<'a> FieldsHelper<'a> {
    fn new(body: &Body) -> FieldsHelper<'a>{
        FieldsHelper {
            body
        }
    }

    fn is_struct(&self) -> bool {
        match self.body {
            Body::Struct(ref variantData) => true,
            _ => false
        }
    }
}

