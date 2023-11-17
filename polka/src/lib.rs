#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use std::{collections::HashSet, fmt::Display};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Symbol(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::Symbol(sym) => write!(f, "'{}", sym),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Interpreter {
    stack: Vec<Value>,
    variables: Vec<(String, Value)>,
    first: HashSet<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            variables: Vec::new(),
            first: HashSet::new(),
        }
    }

    pub fn eval(&mut self, expr: &str) {
        let tokens = expr.split_ascii_whitespace().collect::<Vec<_>>();

        for t in tokens {
            if t.as_bytes()[0].is_ascii_digit() {
                self.stack.push(Value::Number(t.parse().unwrap()));
            } else if ["+", "-", "/", "*"].contains(&t) {
                match (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
                    (Value::Number(a), Value::Number(b)) => {
                        self.stack.push(Value::Number(self.operation(t, a, b)))
                    }
                    (_, _) => panic!("cannot operate on non-numeric values"),
                }
            } else if t.as_bytes()[0] == b'\'' {
                let var = t[1..].to_string();
                self.stack.push(Value::Symbol(var.clone()));
                if !self.first.contains(&var) {
                    self.variables.push((var.clone(), Value::Number(0.)));
                }
                self.first.insert(var);
            } else if t == "set" {
                let var = self.stack.pop().unwrap();
                match var {
                    Value::Number(_) => panic!("cannot set value to numeric value"),
                    Value::Symbol(var) => {
                        let value = self.stack.pop().unwrap();
                        let mut i = 0;
                        for (k, _) in &self.variables {
                            if *k == var {
                                break;
                            }
                            i += 1;
                        }
                        self.variables[i] = (var, value);
                    }
                }
            } else if t.as_bytes()[0] == b'$' {
                let var = t[1..].to_string();
                for (k, v) in &self.variables {
                    if *k == var {
                        self.stack.push(v.clone());
                    }
                }
            } else {
                panic!("unexpected token");
            }
        }
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    fn operation(&self, op: &str, a: f64, b: f64) -> f64 {
        match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => a / b,
            _ => panic!("unexpected token"),
        }
    }
}
