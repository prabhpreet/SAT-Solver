//CNF Definitions
use std::{collections::{HashMap, HashSet}, vec, sync::Arc};

use crate::dimacs::DimacsCnf;

//Literal identified by its unique name
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Literal(String);

impl Literal {
    pub fn new(name: String) -> RefLiteral {
        RefLiteral::new(name)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct RefLiteral(Arc<Literal>);

impl RefLiteral {
    pub fn new(name: String) -> RefLiteral {
        RefLiteral(Arc::new(Literal(name)))
    }

    pub fn positive(&self) -> SignedLiteral {
        SignedLiteral::Literal(self.clone())
    }

    pub fn complement(&self) -> SignedLiteral {
        SignedLiteral::Complement(self.clone())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum SignedLiteral {
    Literal(RefLiteral),
    Complement(RefLiteral),
}

impl SignedLiteral {
    pub fn literal(&self) -> RefLiteral {
        match self {
            SignedLiteral::Literal(literal) => literal.clone(),
            SignedLiteral::Complement(literal) => literal.clone(),
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
            SignedLiteral::Complement(_) => value.negate(),
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
pub struct Assignments(HashMap<RefLiteral, LiteralValue>);

impl Assignments {
    pub fn new() -> Assignments {
        Assignments(HashMap::new())
    }

    pub fn assign(&mut self, literal: RefLiteral, value: LiteralValue) -> &mut Self {
        self.0.insert(literal, value);
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (&RefLiteral, &LiteralValue)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    literals: HashSet<SignedLiteral>,
}

#[derive(Debug, PartialEq)]
pub enum ClauseValue {
    True,
    False,
    Clause(ClauseRef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClauseRef(Arc<Clause>);

impl ClauseValue {
    pub fn is_true(&self) -> bool {
        match self {
            ClauseValue::True => true,
            _ => false,
        }
    }

    pub fn is_false(&self) -> bool {
        match self {
            ClauseValue::False => true,
            _ => false,
        }
    }

    pub fn is_clause(&self) -> bool {
        match self {
            ClauseValue::Clause(_) => true,
            _ => false,
        }
    }
}

impl Clause {
    pub fn new() -> ClauseRef {
        ClauseRef::new()
    }
}

pub struct ClauseBuilder {
    clause: Clause,
}

impl ClauseBuilder {
    pub fn new() -> ClauseBuilder {
        ClauseBuilder {
            clause: Clause{ literals: HashSet::new() },
        }
    }

    pub fn add_literal(mut self, literal: SignedLiteral) -> Self {
        self.clause.literals.insert(literal);
        self
    }

    pub fn build(self) -> ClauseRef {
        ClauseRef(Arc::new(self.clause))
    }
}

impl ClauseRef {
    pub fn new() -> ClauseRef {
        ClauseRef(Arc::new(Clause{ literals: HashSet::new() }))
    }

    pub fn signed_literal(&self) -> impl Iterator<Item = &SignedLiteral> {
        self.0.literals.iter()
    }

    pub fn evaluate(self, assignments: &Assignments) -> ClauseValue {
        let mut mark_for_removal = vec![];
        for literal in self.0.literals.iter() {
            match literal.evaluate(assignments) {
                // C is true if l in C st l is true
                LiteralValue::True => {
                    return ClauseValue::True;
                }
                // Otherwise C is unassigned
                LiteralValue::Unassigned => { }

                // C is false if for each l in C, l is false
                LiteralValue::False => {
                    mark_for_removal.push(literal.clone());
                }
            }
        }

        if !mark_for_removal.is_empty() {
            let mut deep_clone = (*self.0).clone();
            for literal in mark_for_removal.iter() {
                deep_clone.literals.remove(&literal);
            }

            if deep_clone.literals.is_empty() {
                ClauseValue::False
            } else {
                ClauseValue::Clause(ClauseRef(Arc::new(deep_clone)))
            }
        }
        else {
            ClauseValue::Clause(self)
        }
    }

    pub fn is_unit_clause(&self) -> Option<SignedLiteral> {
        //C is an unit clause under m if a literal l in C is unassigned and the rest are false
        // l is a unit literal
        if self.0.literals.len() == 1 {
            self.0.literals.iter().next().cloned()
        }
        else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CNF {
    clauses: Vec<ClauseRef>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CNFValue {
    SAT,
    UNSAT,
    Formula(CNF),
}

impl CNFValue {
    pub fn unwrap(self) -> Satisfiability {
        match self {
            CNFValue::SAT => Satisfiability::SAT,
            CNFValue::UNSAT => Satisfiability::UNSAT,
            CNFValue::Formula(_) => panic!("Unresolved satisfiability"),
        }
    }
    
    pub fn evaluate(self, assignments: &Assignments) -> CNFValue {
        match self {
            CNFValue::Formula(f) => f.evaluate(assignments),
            _ => self,
        }
    }
}


impl CNF {
    pub fn new() -> CNF {
        CNF { clauses: Vec::new() }
    }

    pub fn add_clause(mut self, clause: ClauseRef) -> Self {
        self.clauses.push(clause);
        self
    }

    pub fn clauses(&self) -> impl Iterator<Item = &ClauseRef> {
        self.clauses.iter()
    }

    pub fn evaluate(self, assignments: &Assignments) -> CNFValue{
        let mut clauses = vec![];        
        for clause in self.clauses.into_iter() {
            match clause.evaluate(assignments) {
                // CNF F is false if there is C in F st C is false
                ClauseValue::False => {
                    return CNFValue::UNSAT;
                }
                // Otherwise CNF F is unassigned
                ClauseValue::Clause(c) => {
                    clauses.push(c);
                }
                //CNF F is true if for each C in F, C is true
                ClauseValue::True => {}
            }
        }
        if clauses.is_empty() {
            CNFValue::SAT
        } else {
            CNFValue::Formula(CNF { clauses })
        }
    }

    pub fn iter_literals(&self) -> impl Iterator<Item = &SignedLiteral> {
        self.clauses.iter().flat_map(|c| c.signed_literal())
    }

}

impl From<DimacsCnf> for CNF {
    fn from(dimacs_cnf: DimacsCnf) -> Self {
        let mut cnf = CNF::new();
        for clause in dimacs_cnf.clauses() {
            let mut c = ClauseBuilder::new();
            for literal in clause.iter() {
                let lname = literal.abs().to_string();
                if *literal > 0 {
                    c = c.add_literal(Literal::new(lname).positive());
                } else {
                    c = c.add_literal(Literal::new(lname).complement());
                }
            }
            cnf = cnf.add_clause(c.build());
        }
        cnf
    }
}

#[derive(Debug, PartialEq)]
pub enum Satisfiability {
    SAT,
    UNSAT,
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

        let c1 = ClauseBuilder::new()
            .add_literal(p1.complement())
            .add_literal(p2.positive()).build();
        let c2 = ClauseBuilder::new()
            .add_literal(p1.complement())
            .add_literal(p3.positive())
            .add_literal(p5.positive()).build();
        let c3 = ClauseBuilder::new()
            .add_literal(p2.complement())
            .add_literal(p4.positive()).build();
        let c4 = ClauseBuilder::new()
            .add_literal(p3.complement())
            .add_literal(p4.complement()).build();
        let c5 = ClauseBuilder::new()
            .add_literal(p1.positive())
            .add_literal(p5.positive())
            .add_literal(p2.complement()).build();
        let c6 = ClauseBuilder::new()
            .add_literal(p2.positive())
            .add_literal(p3.positive()).build();
        let c7 = ClauseBuilder::new()
            .add_literal(p2.positive())
            .add_literal(p3.complement())
            .add_literal(p7.positive()).build();
        let c8 = ClauseBuilder::new()
            .add_literal(p6.positive())
            .add_literal(p5.complement()).build();

        {
            let mut assignments = Assignments::new();
            assignments.assign(p1.clone(),LiteralValue::False);

            assert_eq!(c1.clone().evaluate(&assignments),ClauseValue::True);
        }  

        {
            let mut assignments = Assignments::new();
            assignments.assign(p2.clone(),LiteralValue::False);

            let c = c1.clone().evaluate(&assignments);
            println!("c: {:?}",c);
            match c {
                ClauseValue::Clause(c) => {
                    assert_eq!( c.is_unit_clause(), Some(p1.complement()));
                }
                _ => panic!("Expected clause"),
            }
        }  

        let _formula = CNF::new()
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
