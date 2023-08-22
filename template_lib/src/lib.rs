pub fn template_hello(person: &str) {
    println!("Hello, {:?}", person);
}

#[cfg(test)]
mod tests {
    use super::*;
}