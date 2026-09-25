trait MyNotDynTrait { // Is not dyn-compatible (return Self)
    fn make(&self) ->Self ;
}

trait MyDynTrait {  // Is dyn-compatible
    fn name(&self) ->&str ;
}

#[derive(Debug)]
struct S1 {
    v:  String
}

impl MyNotDynTrait for S1 {
    fn make(&self) ->Self {
        Self { 
            v: self.v.clone() 
        }
    }
}

struct S2 {
    v: String
}

impl MyDynTrait for S2 {
    fn name(&self) ->&str {
        self.v.as_str()
        // или  так:
        //&self.v
    }
}

fn use_not_dyn_tr<T>(v: &T) 
    where T: std::fmt::Debug + MyNotDynTrait
{
    println!("not dyn trait: {:?}", v.make()) ;
}

fn user_dyn_trait(v: &dyn MyDynTrait) {
    println!("dyn trait: {}", v.name()) ;
}

fn main() {
    let v_not_dyn = S1{v: "Not Dynamic Trait".to_string()} ;
    use_not_dyn_tr(&v_not_dyn); // Out: not dyn trait: S1 { v: "Not Dynamic Trait" }

    let v_dyn_tr = S2{v: "Dynamic Trait".to_string()} ;
    user_dyn_trait(&v_dyn_tr);  // Out: Dynamic Trait
}
