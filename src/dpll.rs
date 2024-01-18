use crate::{
    definitions::{Assignments, CNFValue, LiteralValue, Satisfiability, CNF},
    Solver,
};
use rand::Rng;

pub struct DPLLSolver {
    formula: CNF,
}

impl Solver for DPLLSolver {
    fn new(formula: CNF) -> Self {
        DPLLSolver { formula }
    }
    fn solve(self) -> Satisfiability {
        self.dpll_recursive(CNFValue::Formula(self.formula.clone()), Assignments::new())
            .unwrap()
    }
}

impl DPLLSolver {
    /*
        DPLL(F,m):
        Input: CNF, partial assigment m
        Output: SAT/UNSAT
    */
    fn dpll_recursive(&self, formula: CNFValue, m: Assignments) -> CNFValue {
        match formula.evaluate(&m) {
            CNFValue::Formula(f) => {
                
                let unassigned_literals = f.unassigned_literals(&m);

                let mut m = Assignments::new();
                //Unit propogation- unit p becomes a unit literal for some clause
                for clause in f.clauses() {
                    for l in unassigned_literals.iter() {
                        if clause.is_unit_clause(l.clone().positive()) {
                            println!(
                                "Unit propogation: Clause {:?}, {:?}",
                                clause,
                                l.clone().positive()
                            );
                            m.assign(l.clone(), LiteralValue::True);
                            return self.dpll_recursive(CNFValue::Formula(f.clone()), m);
                        }
                    }
                }

                for clause in f.clauses() {
                    for l in unassigned_literals.iter() {
                        if clause.is_unit_clause(l.clone().negated()) {
                            println!(
                                "Unit propogation: Clause {:?}, {:?}",
                                clause,
                                l.clone().negated()
                            );
                            m.assign(l.clone(), LiteralValue::False);
                            return self.dpll_recursive(CNFValue::Formula(f.clone()), m);
                        }
                    }
                }

                //Decision: If at this point in time we don't find a unit literal or reach a base case, let's make a decision

                //First decision: Choose an unassigned literal p and a random bit b in {0,1} and check for satisfiability
                let p = unassigned_literals
                    .iter()
                    .nth(rand::thread_rng().gen_range(0..unassigned_literals.len()))
                    .unwrap();
                let value = if rand::thread_rng().gen_bool(0.5) {
                    LiteralValue::True
                } else {
                    LiteralValue::False
                };

                println!("Set {:?} to {:?}", p, value);
                let mut m1 = m.clone();
                m1.assign(p.clone(), value);
                if self.dpll_recursive(CNFValue::Formula(f.clone()),m1) == CNFValue::SAT {
                    println!("SAT: {:?}", m);
                    return CNFValue::SAT;
                }
                // Let's backtrack in case the first decision doesn't work out
                else {
                    let mut m2 = m.clone();
                    m2.assign(p.clone(), value.negate());
                    return self.dpll_recursive(CNFValue::Formula(f.clone()),m2);
                }
            }

            //Base cases: if F is satisfied/unsatisfied by assignments
            v => v,
        }
    }
}
