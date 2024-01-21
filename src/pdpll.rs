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
            depth_par_factor: self.par_factor,
        })
    }
}

pub struct PDPLLSolver {
    formula: CNF,
    depth_par_factor: usize,
}

impl Solver for PDPLLSolver {
    fn solve(&mut self) -> Satisfiability {
        let func = |f: CNF| f.mom(1);
        let vdi = if true {
            VDI::LazyLiteralList(func)
        } else {
            VDI::RandomDecision
        };
        let di = DI::<_>{
            remaining_depth: self.depth_par_factor,
            variable: vdi,
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

#[derive(Debug,PartialEq)]
struct DI<F: Fn(CNF)->Vec<RefLiteral>> {
    variable: VDI<F>,
    remaining_depth: usize,
}

#[derive(Debug,PartialEq)]
//Next iteration decision instruction for choosing variable to assign
enum VDI<F: Fn(CNF)->Vec<RefLiteral>> {
    RandomDecision,
    LiteralList(Vec<RefLiteral>),
    LazyLiteralList(F),
}



impl PDPLLSolver {
    const VALUES: [LiteralValue;2] = [LiteralValue::True, LiteralValue::False];
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
                        DI::<F>{
                            remaining_depth:di.remaining_depth,
                            variable:VDI::RandomDecision,
                        }
                    );
                }

                //Decision: If at this point in time we don't find a unit literal or reach a base case, let's make a decision
                //First decision: Choose an unassigned literal p and a random bit b in {0,1} and check for satisfiability
                let p = match di.variable {
                    VDI::RandomDecision => f.iter_literals()
                        .choose(&mut thread_rng()).unwrap().literal(),
                    VDI::LiteralList(ref l) => {
                        if let Some(l) = l.first() {
                            l.to_owned()
                        } else {
                            f.iter_literals()
                                .choose(&mut thread_rng()).unwrap().literal()
                        }
                    },
                    VDI::LazyLiteralList(ref func) => {
                        if let Some(l) = func(f.clone()).first() {
                            l.to_owned()
                        } else {
                            f.iter_literals()
                                .choose(&mut thread_rng()).unwrap().literal()
                        }
                    },
                };

                if di.remaining_depth > 0 {
                    return Self::VALUES 
                        .iter()
                        .par_bridge()
                        .find_map_any(|&value| {
                            if self.dpll_recursive::<F>(
                                CNFValue::Formula(f.clone()),
                                Assignments::new().assign(p.clone(), value).to_owned(),
                                DI::<F>{
                                    remaining_depth:di.remaining_depth-1,
                                    variable: VDI::RandomDecision,
                                }
                            ) == CNFValue::SAT
                            {
                                debug!("SAT: {:?}", &m);
                                return Some(CNFValue::SAT);
                            }
                            else {
                                None
                            }
                        })
                        .unwrap_or(CNFValue::UNSAT);
                }
                else {
                    let value = LiteralValue::True;
                    debug!("Set {:?} to {:?}", p, value);
                    if self.dpll_recursive::<F>(
                        CNFValue::Formula(f.clone()),
                        Assignments::new().assign(p.clone(), value).to_owned(),
                        DI::<F>{
                            remaining_depth:di.remaining_depth,
                            variable: VDI::RandomDecision,
                        }
                    ) == CNFValue::SAT
                    {
                        debug!("SAT: {:?}", &m);
                        return CNFValue::SAT;
                    }
                    // Let's backtrack in case the first decision doesn't work out
                    else {
                        return self.dpll_recursive::<F>(
                            CNFValue::Formula(f.clone()),
                            Assignments::new()
                                .assign(p.clone(), value.negate())
                                .to_owned(),
                            DI::<F>{
                                remaining_depth:di.remaining_depth,
                                variable: VDI::RandomDecision,
                            }
                        );
                    }
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
    crate::tests::sat_tests!(PDPLLSolverBuilder::new(4));
}
