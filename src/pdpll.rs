use crate::{
    definitions::{
        Assignments, CNFValue, LiteralValue, RefLiteral, Satisfiability, SignedLiteral, CNF,
    },
    Solver, SolverBuilder,
};
use log::debug;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

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
        let func = |f: CNF| f.mom(self.par_factor);
        let di = if true {
            DI::LazyParallelizeLiterals((func, self.par_factor))
        } else {
            DI::RandomlyParallelize(self.par_factor)
        };

        //Pure literal elimination- if a literal l only appears as positive/negative, assign it to true/false
        //to strive for satisfiability
        let mut m = Assignments::new();
        self.formula.pure_literals().iter().for_each(|l| {
            let value = match l {
                SignedLiteral::Id(_) => LiteralValue::True,
                SignedLiteral::Not(_) => LiteralValue::False,
            };
            debug!("Pure literal elimination: Literal {:?}, {:?}", l, value);
            m.assign(l.literal(), value);
        });

        self.dpll_recursive(CNFValue::Formula(self.formula.clone()), m, di)
            .unwrap()
    }
}

#[derive(Debug, PartialEq)]
//Next iteration decision instruction
enum DI<F: Fn(CNF) -> Vec<RefLiteral>> {
    NoParallel,
    //Randomly choose a set of literals as count
    RandomlyParallelize(usize),
    //Parallelize a set of literals
    ParallelizeLiterals(Vec<RefLiteral>),
    LazyParallelizeLiterals((F, usize)),
}

impl PDPLLSolver {
    /*
        DPLL(F,m):
        Input: CNF, partial assigment m
        Output: SAT/UNSAT
    */
    fn dpll_recursive<F: Fn(CNF) -> Vec<RefLiteral>>(
        &self,
        formula: CNFValue,
        m: Assignments,
        di: DI<F>,
    ) -> CNFValue {
        match formula.evaluate(&m) {
            CNFValue::Formula(f) => {
                //Unit clause propogation- unit p becomes a unit literal for some clause
                let m = f.clauses().find_map(|clause| {
                    if let Some(l) = clause.is_unit_clause() {
                        let value = match l {
                            SignedLiteral::Id(_) => LiteralValue::True,
                            SignedLiteral::Not(_) => LiteralValue::False,
                        };
                        debug!("Unit propogation: Clause {:?}, {:?}", clause, value);
                        return Some(Assignments::new().assign(l.literal(), value).to_owned());
                    }
                    None
                });

                if let Some(m) = m {
                    return self.dpll_recursive::<F>(
                        CNFValue::Formula(f.clone()),
                        m,
                        DI::NoParallel,
                    );
                }

                //Decision: If at this point in time we don't find a unit literal or reach a base case, let's make a decision
                //First decision: Choose an unassigned literal p and a random bit b in {0,1} and check for satisfiability
                let p = match di {
                    DI::ParallelizeLiterals(value) => {
                        if value.is_empty() {
                            vec![f
                                .iter_literals()
                                .map(|l| l.literal())
                                .choose(&mut thread_rng())
                                .unwrap()]
                        } else {
                            value
                        }
                    }
                    DI::LazyParallelizeLiterals((func, par_factor)) => {
                        let value = func(f.clone());
                        if value.is_empty() {
                            f.iter_literals()
                                .map(|l| l.literal())
                                .choose_multiple(&mut thread_rng(), par_factor)
                        } else {
                            value
                        }
                    }
                    DI::RandomlyParallelize(par_factor) => f
                        .iter_literals()
                        .map(|l| l.literal())
                        .choose_multiple(&mut thread_rng(), par_factor),
                    DI::NoParallel => vec![f
                        .iter_literals()
                        .map(|l| l.literal())
                        .choose(&mut thread_rng())
                        .unwrap()],
                };

                return p
                    .iter()
                    .par_bridge()
                    .find_map_any(|p| {
                        //Positive bias
                        let value = LiteralValue::True;

                        if self.dpll_recursive::<F>(
                            CNFValue::Formula(f.clone()),
                            Assignments::new().assign(p.clone(), value).to_owned(),
                            DI::NoParallel,
                        ) == CNFValue::SAT
                        {
                            debug!("SAT: {:?}", &m);
                            return Some(CNFValue::SAT);
                        }
                        // Let's backtrack in case the first decision doesn't work out
                        else {
                            return Some(
                                self.dpll_recursive::<F>(
                                    CNFValue::Formula(f.clone()),
                                    Assignments::new()
                                        .assign(p.clone(), value.negate())
                                        .to_owned(),
                                    DI::NoParallel,
                                )
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
