//CNF Definitions
use std::collections::{HashMap, HashSet};

//Literal identified by its unique name
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Literal(String);

impl Literal {
    pub fn new(name: String) -> Literal {
        Literal(name)
    }

    pub fn positive(&self) -> SignedLiteral {
        SignedLiteral::Literal(self.clone())
    }

    pub fn negated(&self) -> SignedLiteral {
        SignedLiteral::NegatedLiteral(self.clone())
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SignedLiteral {
    Literal(Literal),
    NegatedLiteral(Literal),
}

impl SignedLiteral {
    pub fn literal(&self) -> Literal {
        match self {
            SignedLiteral::Literal(literal) => literal.clone(),
            SignedLiteral::NegatedLiteral(literal) => literal.clone(),
        }
    }

    pub fn evaluate(&self, assignments: &Assignments) -> LiteralValue {
        let value = assignments
            .0
            .get(&self.literal())
            .unwrap_or(&LiteralValue::Unassigned)
            .clone();

        match self {
            SignedLiteral::Literal(_) => value,
            SignedLiteral::NegatedLiteral(_) => value.negate(),
        }
    }
}

#[derive(Debug,PartialEq, Eq, Clone, Copy)]
pub enum LiteralValue {
    True,
    False,
    Unassigned,
}

impl LiteralValue {
    pub fn negate(&self) -> LiteralValue {
        match self {
            LiteralValue::True => LiteralValue::False,
            LiteralValue::False => LiteralValue::True,
            LiteralValue::Unassigned => LiteralValue::Unassigned,
        }
    }
}

#[derive(Debug,PartialEq, Clone)]
pub struct Assignments(HashMap<Literal, LiteralValue>);

impl Assignments {
    pub fn new() -> Assignments {
        Assignments(HashMap::new())
    }

    pub fn assign(&mut self, literal: Literal, value: LiteralValue) {
        self.0.insert(literal, value);
    }
}

#[derive(Debug)]
pub struct Clause {
    literals: Vec<SignedLiteral>,
}

impl Clause {
    pub fn new() -> Clause {
        Clause { literals: Vec::new() }
    }

    pub fn add_literal(mut self, literal: SignedLiteral) -> Self {
        self.literals.push(literal);
        self
    }

    pub fn literals(&self) -> impl Iterator<Item = &SignedLiteral> {
        self.literals.iter()
    }

    fn evaluate(&self, assignments: &Assignments) -> LiteralValue {
        for literal in self.literals.iter() {
            match literal.evaluate(assignments) {
                // C is true if l in C st l is true
                LiteralValue::True => {
                    return LiteralValue::True;
                }
                // Otherwise C is unassigned
                LiteralValue::Unassigned => {
                    return LiteralValue::Unassigned;
                }

                // C is false if for each l in C, l is false
                LiteralValue::False => {}
            }
        }
        LiteralValue::False
    }

    pub fn is_unit_clause(&self, literal: SignedLiteral, assignments: &Assignments) -> bool {
        //C is an unit clause under m if a literal l in C is unassigned and the rest are false
        // l is a unit literal

        match assignments.0.get(&literal.literal()) {
            Some(&LiteralValue::Unassigned) | None => {
                let mut unassigned_count = 0;

                for l in self.literals.iter() {
                    if l == &literal {
                        unassigned_count += 1;
                        continue;
                    } else {
                        match l.evaluate(assignments) {
                            LiteralValue::Unassigned => {
                                //Other unassigned literals
                                return false;
                            }
                            LiteralValue::False => {}
                            LiteralValue::True => {
                                return false;
                            }
                        }
                    }
                }

                if unassigned_count == 1 {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct CNF {
    clauses: Vec<Clause>,
}

impl CNF {
    pub fn new() -> CNF {
        CNF { clauses: Vec::new() }
    }

    pub fn add_clause(mut self, clause: Clause) -> Self {
        self.clauses.push(clause);
        self
    }

    pub fn clauses(&self) -> impl Iterator<Item = &Clause> {
        self.clauses.iter()
    }

    pub fn evaluate(&self, assignments: &Assignments) -> LiteralValue {
        for clause in self.clauses.iter() {
            match clause.evaluate(assignments) {
                // CNF F is false if there is C in F st C is false
                LiteralValue::False => {
                    return LiteralValue::False;
                }
                // Otherwise CNF F is unassigned
                LiteralValue::Unassigned => {
                    return LiteralValue::Unassigned;
                }
                //CNF F is true if for each C in F, C is true
                LiteralValue::True => {}
            }
        }
        LiteralValue::True
    }

    pub fn unassigned_literals(&self, assignments: &Assignments) -> HashSet<Literal> {
        let mut unassigned_literals = HashSet::new();
        for clause in self.clauses.iter() {
            for literal in clause.literals.iter() {
                match literal.evaluate(assignments) {
                    LiteralValue::Unassigned => {
                        unassigned_literals.insert(literal.literal().clone());
                    }
                    _ => {}
                }
            }
        }
        unassigned_literals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause() {
                /*
           c1 = (¬p1 ∨ p2)
           c2 = (¬p1 ∨ p3 ∨ p5)
           c3 = (¬p2 ∨ p4)
           c4 = (¬p3 ∨ ¬p4)
           c5 = (p1 ∨ p5 ∨ ¬p2)
           c6 = (p2 ∨ p3)
           c7 = (p2 ∨ ¬p3 ∨ p7)
           c8 = (p6 ∨ ¬p5)
        */

        let p1 = Literal::new("p1".to_string());
        let p2 = Literal::new("p2".to_string());
        let p3 = Literal::new("p3".to_string());
        let p4 = Literal::new("p4".to_string());
        let p5 = Literal::new("p5".to_string());
        let p6 = Literal::new("p6".to_string());
        let p7 = Literal::new("p7".to_string());

        let c1 = Clause::new()
            .add_literal(p1.negated())
            .add_literal(p2.positive());
        let c2 = Clause::new()
            .add_literal(p1.negated())
            .add_literal(p3.positive())
            .add_literal(p5.positive());
        let c3 = Clause::new()
            .add_literal(p2.negated())
            .add_literal(p4.positive());
        let c4 = Clause::new()
            .add_literal(p3.negated())
            .add_literal(p4.negated());
        let c5 = Clause::new()
            .add_literal(p1.positive())
            .add_literal(p5.positive())
            .add_literal(p2.negated());
        let c6 = Clause::new()
            .add_literal(p2.positive())
            .add_literal(p3.positive());
        let c7 = Clause::new()
            .add_literal(p2.positive())
            .add_literal(p3.negated())
            .add_literal(p7.positive());
        let c8 = Clause::new()
            .add_literal(p6.positive())
            .add_literal(p5.negated());

        {
            let mut assignments = Assignments::new();
            assignments.assign(p1.clone(),LiteralValue::False);

            assert_eq!(c1.evaluate(&assignments), LiteralValue::True);
            assert!(!c1.is_unit_clause(p2.positive(), &assignments));

        }  

        {
            let mut assignments = Assignments::new();
            assignments.assign(p2.clone(),LiteralValue::False);

            assert_eq!(c1.evaluate(&assignments), LiteralValue::Unassigned);
            assert!(c1.is_unit_clause(p1.negated(), &assignments));
            assert!(!c1.is_unit_clause(p1.positive(), &assignments));
        }  

        let formula = CNF::new()
            .add_clause(c1)
            .add_clause(c2)
            .add_clause(c3)
            .add_clause(c4)
            .add_clause(c5)
            .add_clause(c6)
            .add_clause(c7)
            .add_clause(c8);


    }

}
