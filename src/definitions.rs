//CNF Definitions
use std::{collections::{HashMap, HashSet}, vec, sync::Arc, cell::Ref};

use log::debug;

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

    pub fn identity(&self) -> SignedLiteral {
        SignedLiteral::Id(self.clone())
    }

    pub fn not(&self) -> SignedLiteral {
        SignedLiteral::Not(self.clone())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum SignedLiteral {
    Id(RefLiteral),
    Not(RefLiteral),
}

impl SignedLiteral {
    pub fn literal(&self) -> RefLiteral {
        match self {
            SignedLiteral::Id(literal) => literal.clone(),
            SignedLiteral::Not(literal) => literal.clone(),
        }
    }

    pub fn complement(&self) -> SignedLiteral {
        match self {
            SignedLiteral::Id(literal) => literal.not(),
            SignedLiteral::Not(literal) => literal.identity(),
        }
    }

    pub fn evaluate(&self, assignments: &Assignments) -> LiteralValue {
        let value = assignments
            .0
            .get(&self.literal())
            .unwrap_or(&LiteralValue::Unassigned)
            .clone();

        match self {
            SignedLiteral::Id(_) => value,
            SignedLiteral::Not(_) => value.negate(),
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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

    //Insert in a sorted manner
    pub fn add_clause(mut self, clause: ClauseRef) -> Self {
        //Find index
        let index = self.clauses.iter().position(|c| c.0.literals.len() > clause.0.literals.len()).unwrap_or(0);
        self.clauses.insert(index, clause);
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

    pub fn pure_literals(&self) -> HashSet<SignedLiteral> {
        let mut pure_literals = HashSet::new();
        let mut impure_literals = HashSet::new();
        self.iter_literals().for_each(|l| {
            let abs_literal = l.literal().identity();
            let complement = l.complement();
            if impure_literals.contains(&abs_literal) { } else {
                if pure_literals.contains(&complement) {
                    pure_literals.remove(&complement);
                    impure_literals.insert(abs_literal);
                }
                else {
                    pure_literals.insert(l.clone());
                }
            }
        });
        pure_literals
    }

    //Most occurences in clauses of minimum length
    pub fn mom(&self, max_literals: usize) -> Vec<RefLiteral> {
        //Find the clause with minimum length
        let min_len_clause = self.clauses.iter().fold(None, |min_clause: Option<&ClauseRef>, clause| {
            if let Some(min_clause) = min_clause {
                if clause.0.literals.len() < min_clause.0.literals.len() {
                    Some(clause)
                } else {
                    Some(min_clause)
                }
            }
            else {
                Some(clause)
            }
        });

        //let min_len_clause = self.clauses.first();

        if let Some(min_len_clause) = min_len_clause  {
            let mut literal_counts = HashMap::new();

            for signed_literal in self.iter_literals() {
                let entry = literal_counts.entry(signed_literal.literal()).or_insert(0usize); 
                *entry += 1;
            }

            //Find the literal with the maximum count
            let mut literal_counts : Vec<(RefLiteral,usize)> = min_len_clause.0.literals.iter().map(|literal| {
                let literal = literal.literal();
                let count = literal_counts.get(&literal).unwrap_or(&0usize);
                (literal,*count)
            }).collect();
            literal_counts.sort_by(|a,b| b.1.cmp(&a.1));
            literal_counts.into_iter().take(max_literals).map(|(literal,_)| literal).collect()
        }
        else {
            vec![]
        }
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
                    c = c.add_literal(Literal::new(lname).identity());
                } else {
                    c = c.add_literal(Literal::new(lname).not());
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
            .add_literal(p1.not())
            .add_literal(p2.identity()).build();
        let c2 = ClauseBuilder::new()
            .add_literal(p1.not())
            .add_literal(p3.identity())
            .add_literal(p5.identity()).build();
        let c3 = ClauseBuilder::new()
            .add_literal(p2.not())
            .add_literal(p4.identity()).build();
        let c4 = ClauseBuilder::new()
            .add_literal(p3.not())
            .add_literal(p4.not()).build();
        let c5 = ClauseBuilder::new()
            .add_literal(p1.identity())
            .add_literal(p5.identity())
            .add_literal(p2.not()).build();
        let c6 = ClauseBuilder::new()
            .add_literal(p2.identity())
            .add_literal(p3.identity()).build();
        let c7 = ClauseBuilder::new()
            .add_literal(p2.identity())
            .add_literal(p3.not())
            .add_literal(p7.identity()).build();
        let c8 = ClauseBuilder::new()
            .add_literal(p6.identity())
            .add_literal(p5.not()).build();

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
                    assert_eq!( c.is_unit_clause(), Some(p1.not()));
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
