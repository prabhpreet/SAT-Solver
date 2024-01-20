use crate::{
    definitions::{Assignments, CNFValue, LiteralValue, Satisfiability, CNF, SignedLiteral},
    Solver, SolverBuilder,
};
use rand::{Rng, seq::IteratorRandom, rngs, thread_rng};
use log::debug;

pub struct DPLLSolverBuilder {
}

impl DPLLSolverBuilder {
    pub fn new() -> Self {
        DPLLSolverBuilder { }
    }

}

impl SolverBuilder for DPLLSolverBuilder {
    fn build(self, formula: CNF) -> Box<dyn Solver> {
        Box::new(DPLLSolver{formula})
    }
}

pub struct DPLLSolver {
    formula: CNF,
}

impl Solver for DPLLSolver {
    fn solve(&mut self) -> Satisfiability {
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
                //Unit propogation- unit p becomes a unit literal for some clause
                let m = || -> Option<Assignments> {
                    for clause in f.clauses() {
                        if let Some(l) = clause.is_unit_clause() {
                            let value = match l {
                                SignedLiteral::Literal(_) => LiteralValue::True,
                                SignedLiteral::Complement(_) => LiteralValue::False,
                            };
                            debug!("Unit propogation: Clause {:?}, {:?}", clause, value);
                            return Some(Assignments::new().assign(l.literal(), value).to_owned());
                        }
                    }
                    None
                }();

                if let Some(m) = m {
                    return self.dpll_recursive(CNFValue::Formula(f.clone()), m);
                }

                //Decision: If at this point in time we don't find a unit literal or reach a base case, let's make a decision

                //First decision: Choose an unassigned literal p and a random bit b in {0,1} and check for satisfiability
                let p = f.iter_literals()
                    .choose(&mut thread_rng()).unwrap().literal();
                let value = if rand::thread_rng().gen_bool(0.5) {
                    LiteralValue::True
                } else {
                    LiteralValue::False
                };

                debug!("Set {:?} to {:?}", p, value);
                if self.dpll_recursive(
                    CNFValue::Formula(f.clone()),
                    Assignments::new().assign(p.clone(), value).to_owned(),
                ) == CNFValue::SAT
                {
                    debug!("SAT: {:?}", m);
                    return CNFValue::SAT;
                }
                // Let's backtrack in case the first decision doesn't work out
                else {
                    return self.dpll_recursive(
                        CNFValue::Formula(f.clone()),
                        Assignments::new()
                            .assign(p.clone(), value.negate())
                            .to_owned(),
                    );
                }
            }

            //Base cases: if F is satisfied/unsatisfied by assignments
            v => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::tests::sat_tests!(DPLLSolverBuilder::new());
}