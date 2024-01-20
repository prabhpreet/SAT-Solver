use crate::{
    definitions::{Assignments, CNFValue, LiteralValue, Satisfiability, SignedLiteral, CNF},
    Solver, SolverBuilder,
};
use log::debug;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use rayon::iter::{ParallelBridge, ParallelIterator};

pub struct PDPLLSolverBuilder {
    par_factor: usize,
}

impl PDPLLSolverBuilder {
    pub fn new(par_factor: usize) -> Self {
        PDPLLSolverBuilder { par_factor }
    }
}
impl SolverBuilder for PDPLLSolverBuilder {
    fn build(self, formula: CNF) -> Box<dyn Solver> {
        Box::new(PDPLLSolver {
            formula,
            par_factor: self.par_factor,
        })
    }
}

pub struct PDPLLSolver {
    formula: CNF,
    par_factor: usize,
}

impl Solver for PDPLLSolver {
    fn solve(&mut self) -> Satisfiability {
        self.dpll_recursive(
            CNFValue::Formula(self.formula.clone()),
            Assignments::new(),
            true,
        )
        .unwrap()
    }
}

impl PDPLLSolver {
    /*
        DPLL(F,m):
        Input: CNF, partial assigment m
        Output: SAT/UNSAT
    */
    fn dpll_recursive(&self, formula: CNFValue, m: Assignments, par: bool) -> CNFValue {
        match formula.evaluate(&m) {
            CNFValue::Formula(f) => {
                //Unit propogation- unit p becomes a unit literal for some clause
                let m = f.clauses().par_bridge().find_map_any(|clause| {
                    if let Some(l) = clause.is_unit_clause() {
                        let value = match l {
                            SignedLiteral::Literal(_) => LiteralValue::True,
                            SignedLiteral::Complement(_) => LiteralValue::False,
                        };
                        debug!("Unit propogation: Clause {:?}, {:?}", clause, value);
                        return Some(Assignments::new().assign(l.literal(), value).to_owned());
                    }
                    None
                });

                if let Some(m) = m {
                    return self.dpll_recursive(CNFValue::Formula(f.clone()), m, false);
                }

                //Decision: If at this point in time we don't find a unit literal or reach a base case, let's make a decision

                //First decision: Choose an unassigned literal p and a random bit b in {0,1} and check for satisfiability
                let p = if par {
                    f.iter_literals().map(|l| l.literal())
                        .choose_multiple(&mut thread_rng(), self.par_factor)
                } else {
                    vec![f.iter_literals().map(|l| l.literal())
                        .choose(&mut thread_rng())
                        .unwrap()]
                };

                return p
                    .iter()
                    .par_bridge()
                    .find_map_any(|p| {
                        let value = if thread_rng().gen_bool(0.5) {
                            LiteralValue::True
                        } else {
                            LiteralValue::False
                        };

                        if self.dpll_recursive(
                            CNFValue::Formula(f.clone()),
                            Assignments::new().assign(p.clone(), value).to_owned(),
                            false,
                        ) == CNFValue::SAT
                        {
                            debug!("SAT: {:?}", &m);
                            return Some(CNFValue::SAT);
                        }
                        // Let's backtrack in case the first decision doesn't work out
                        else {
                            return Some(
                                self.dpll_recursive(
                                    CNFValue::Formula(f.clone()),
                                    Assignments::new()
                                        .assign(p.clone(), value.negate())
                                        .to_owned(),
                                    false,
                                ),
                            );
                        }
                    })
                    .unwrap_or(CNFValue::UNSAT);
            }

            //Base cases: if F is satisfied/unsatisfied by assignments
            v => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::tests::sat_tests!(PDPLLSolverBuilder::new(4));
}
