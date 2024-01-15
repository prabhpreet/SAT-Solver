use crate::definitions::{CNF, Assignments, LiteralValue};
use rand::Rng;

#[derive(Debug, PartialEq)]
pub enum Satisfeability {
    SAT,
    UNSAT
}

pub fn dpll(formula: &CNF) -> Satisfeability {
    let m = Assignments::new();
    dpll_recursive(formula, m)
}

/*
    DPLL(F,m):
    Input: CNF, partial assigment m
    Output: SAT/UNSAT
*/

fn dpll_recursive(formula: &CNF, mut m: Assignments) -> Satisfeability  {

    //println!("{:?}", m);

    //Base cases: if F is satisfied by assignments
    if formula.evaluate(&m) == LiteralValue::True {
        println!("SAT Base: {:?}", m);
        return Satisfeability::SAT;
    }

    //If F is unsatisfied by assignments
    if formula.evaluate(&m) == LiteralValue::False {
        println!("UNSAT: {:?}", m);
        return Satisfeability::UNSAT;
    }

    let unassigned_literals = formula.unassigned_literals(&m);

    //Unit propogation- unit p becomes a unit literal for some clause
    for clause in formula.clauses() {
        for l in unassigned_literals.iter() {
            if clause.is_unit_clause(l.clone().positive(), &m) {
                println!("Unit propogation: Clause {:?}, {:?}", clause, l.clone().positive());
                m.assign(l.clone(), LiteralValue::True);
                return dpll_recursive(formula, m);
            }
        }
    }

    for clause in formula.clauses() {
        for l in unassigned_literals.iter() {
            if clause.is_unit_clause(l.clone().negated(), &m) {
                println!("Unit propogation: Clause {:?}, {:?}", clause, l.clone().negated());
                m.assign(l.clone(), LiteralValue::False);
                return dpll_recursive(formula, m);
            }
        }
    }

    //Decision: 
    //Choose an unassigned literal p and a random bit b in {0,1} and check for satisfiability
    let p = unassigned_literals.iter().nth(rand::thread_rng().gen_range(0..unassigned_literals.len())).unwrap();
    let value = if rand::thread_rng().gen_bool(0.5) {LiteralValue::True} else {LiteralValue::False};

    println!("Set {:?} to {:?}", p, value);
    let mut m1 = m.clone();
    m1.assign(p.clone(), value);
    if dpll_recursive(formula, m1) == Satisfeability::SAT {
        println!("SAT: {:?}",m);
        return Satisfeability::SAT;
    }
    else {
        let mut m2 = m.clone();
        m2.assign(p.clone(), value.negate());
        return dpll_recursive(formula, m2);
    }

}