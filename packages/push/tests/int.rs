use proptest::{prop_assert_eq, proptest};
use push::{
    instruction::{Instruction, IntInstruction, IntInstructionError, PushInstructionError},
    push_vm::{push_state::PushState, HasStack, PushInteger},
};
use strum::IntoEnumIterator;

#[test]
fn add() {
    let x = 409;
    let y = 512;
    let mut state = PushState::builder()
        .with_max_stack_size(100)
        .with_program([])
        .unwrap()
        .build();
    state.stack_mut::<PushInteger>().push(y).unwrap();
    state.stack_mut::<PushInteger>().push(x).unwrap();
    let result = IntInstruction::Add.perform(state).unwrap();
    assert_eq!(result.stack::<PushInteger>().size(), 1);
    assert_eq!(*result.stack::<PushInteger>().top().unwrap(), x + y);
}

#[test]
fn add_overflows() {
    let x = 4_098_586_571_925_584_936;
    let y = 5_124_785_464_929_190_872;
    let mut state = PushState::builder()
        .with_max_stack_size(100)
        .with_program([])
        .unwrap()
        .build();
    state.stack_mut::<PushInteger>().push(y).unwrap();
    state.stack_mut::<PushInteger>().push(x).unwrap();
    let result = IntInstruction::Add.perform(state).unwrap_err();
    assert_eq!(result.state().stack::<PushInteger>().size(), 2);
    assert_eq!(
        result.error(),
        &PushInstructionError::from(IntInstructionError::Overflow {
            op: IntInstruction::Add
        })
    );
    assert!(result.is_recoverable());
}

#[test]
fn inc_overflows() {
    let x = PushInteger::MAX;
    let mut state = PushState::builder()
        .with_max_stack_size(100)
        .with_program([])
        .unwrap()
        .build();
    state.stack_mut::<PushInteger>().push(x).unwrap();
    let result = IntInstruction::Inc.perform(state).unwrap_err();
    assert_eq!(result.state().stack::<PushInteger>().size(), 1);
    assert_eq!(
        result.state().stack::<PushInteger>().top().unwrap(),
        &PushInteger::MAX
    );
    assert_eq!(
        result.error(),
        &IntInstructionError::Overflow {
            op: IntInstruction::Inc
        }
        .into()
    );
    assert!(result.is_recoverable());
}

#[test]
fn dec_overflows() {
    let x = PushInteger::MIN;
    let mut state = PushState::builder()
        .with_max_stack_size(100)
        .with_program([])
        .unwrap()
        .build();
    state.stack_mut::<PushInteger>().push(x).unwrap();
    let result = IntInstruction::Dec.perform(state).unwrap_err();
    assert_eq!(result.state().stack::<PushInteger>().size(), 1);
    assert_eq!(
        result.state().stack::<PushInteger>().top().unwrap(),
        &PushInteger::MIN
    );
    assert_eq!(
        result.error(),
        &IntInstructionError::Overflow {
            op: IntInstruction::Dec
        }
        .into()
    );
    assert!(result.is_recoverable());
}

fn all_instructions() -> Vec<IntInstruction> {
    IntInstruction::iter().collect()
}

proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(1_000))]

    #[test]
    fn negate(x in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Negate.perform(state).unwrap();
        prop_assert_eq!(result.stack::<PushInteger>().size(), 1);
        prop_assert_eq!(*result.stack::<PushInteger>().top().unwrap(), -x);
    }

    #[test]
    fn abs(x in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Abs.perform(state).unwrap();
        prop_assert_eq!(result.stack::<PushInteger>().size(), 1);
        prop_assert_eq!(*result.stack::<PushInteger>().top().unwrap(), x.abs());
    }

    #[test]
    fn sqr(x in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Square.perform(state);
        if let Some(x_squared) = x.checked_mul(x) {
            let result = result.unwrap();
            prop_assert_eq!(result.stack::<PushInteger>().size(), 1);
            let output = *result.stack::<PushInteger>().top().unwrap();
            prop_assert_eq!(output, x_squared);
        } else {
            let result = result.unwrap_err();
            assert_eq!(
                result.error(),
                &IntInstructionError::Overflow {
                    op: IntInstruction::Square
                }.into()
            );
            assert!(result.is_recoverable());
            let top_int = result.state().stack::<PushInteger>().top().unwrap();
            prop_assert_eq!(*top_int, x);
        }
    }

    #[test]
    fn add_doesnt_crash(x in proptest::num::i64::ANY, y in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(y).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let _ = IntInstruction::Add.perform(state);
    }

    #[test]
    fn add_adds_or_does_nothing(x in proptest::num::i64::ANY, y in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(y).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Add.perform(state);
        #[allow(clippy::unwrap_used)]
        if let Some(expected_result) = x.checked_add(y) {
            let output = result.unwrap().stack_mut::<PushInteger>().pop().unwrap();
            prop_assert_eq!(output, expected_result);
        } else {
            // This only checks that `x` is still on the top of the stack.
            // We arguably want to confirm that the entire state of the system
            // is unchanged, except that the `Add` instruction has been
            // removed from the `exec` stack.
            let result = result.unwrap_err();
            assert_eq!(
                result.error(),
                &IntInstructionError::Overflow {
                    op: IntInstruction::Add
                }
                .into()
            );
            assert!(result.is_recoverable());
            let top_int = result.state().stack::<PushInteger>().top().unwrap();
            prop_assert_eq!(*top_int, x);
        }
    }

    #[test]
    fn subtract_subs_or_does_nothing(x in proptest::num::i64::ANY, y in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(y).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Subtract.perform(state);
        #[allow(clippy::unwrap_used)]
        if let Some(expected_result) = x.checked_sub(y) {
            let output = result.unwrap().stack_mut::<PushInteger>().pop().unwrap();
            prop_assert_eq!(output, expected_result);
        } else {
            // This only checks that `x` is still on the top of the stack.
            // We arguably want to confirm that the entire state of the system
            // is unchanged, except that the `Add` instruction has been
            // removed from the `exec` stack.
            let result = result.unwrap_err();
            assert_eq!(
                result.error(),
                &IntInstructionError::Overflow {
                    op: IntInstruction::Subtract
                }
                .into()
            );
            assert!(result.is_recoverable());
            let top_int = result.state().stack::<PushInteger>().top().unwrap();
            prop_assert_eq!(*top_int, x);
        }
    }

    #[test]
    fn multiply_muls_or_does_nothing(x in proptest::num::i64::ANY, y in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(y).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Multiply.perform(state);
        #[allow(clippy::unwrap_used)]
        if let Some(expected_result) = x.checked_mul(y) {
            let output = result.unwrap().stack_mut::<PushInteger>().pop().unwrap();
            prop_assert_eq!(output, expected_result);
        } else {
            // This only checks that `x` is still on the top of the stack.
            // We arguably want to confirm that the entire state of the system
            // is unchanged, except that the `Add` instruction has been
            // removed from the `exec` stack.
            let result = result.unwrap_err();
            assert_eq!(
                result.error(),
                &IntInstructionError::Overflow {
                    op: IntInstruction::Multiply
                }
                .into()
            );
            assert!(result.is_recoverable());
            let top_int = result.state().stack::<PushInteger>().top().unwrap();
            prop_assert_eq!(*top_int, x);
        }
    }

    #[test]
    fn protected_divide_zero_denominator(x in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(0).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::ProtectedDivide.perform(state);
        #[allow(clippy::unwrap_used)]
        let output = result.unwrap().stack_mut::<PushInteger>().pop().unwrap();
        // Dividing by zero should always return 1.
        prop_assert_eq!(output, 1);
    }

    #[test]
    fn protected_divide_divs_or_does_nothing(x in proptest::num::i64::ANY, y in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(y).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::ProtectedDivide.perform(state);
        #[allow(clippy::unwrap_used)]
        if let Some(expected_result) = x.checked_div(y) {
            let output = result.unwrap().stack_mut::<PushInteger>().pop().unwrap();
            prop_assert_eq!(output, expected_result);
        } else {
            // This only checks that `x` is still on the top of the stack.
            // We arguably want to confirm that the entire state of the system
            // is unchanged, except that the `Add` instruction has been
            // removed from the `exec` stack.
            let result = result.unwrap_err();
            assert_eq!(
                result.error(),
                &IntInstructionError::Overflow {
                    op: IntInstruction::ProtectedDivide
                }
                .into()
            );
            assert!(result.is_recoverable());
            let top_int = result.state().stack::<PushInteger>().top().unwrap();
            prop_assert_eq!(*top_int, x);
        }
    }

    #[test]
    fn mod_zero_denominator(x in proptest::num::i64::ANY) {
        let mut state =PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(0).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Mod.perform(state);
        #[allow(clippy::unwrap_used)]
        let output = result.unwrap().stack_mut::<PushInteger>().pop().unwrap();
        // Modding by zero should always return 0 since x % x = 0 for all x != 0.
        prop_assert_eq!(output, 0);
    }

    #[test]
    fn mod_rems_or_does_nothing(x in proptest::num::i64::ANY, y in proptest::num::i64::ANY) {
        let mut state =PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(y).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let result = IntInstruction::Mod.perform(state);
        #[allow(clippy::unwrap_used)]
        if let Some(expected_result) = x.checked_rem(y) {
            let output = result.unwrap().stack_mut::<PushInteger>().pop().unwrap();
            prop_assert_eq!(output, expected_result);
        } else if y == 0 {
            let output: i64 = *result.unwrap().stack_mut::<PushInteger>().top().unwrap();
            // Modding by zero should always return 0 since x % x == 0 for all x != 0.
            prop_assert_eq!(output, 0);
        } else {
            // This only checks that `x` is still on the top of the stack.
            // We arguably want to confirm that the entire state of the system
            // is unchanged, except that the `Add` instruction has been
            // removed from the `exec` stack.
            let result = result.unwrap_err();
            assert_eq!(
                result.error(),
                &IntInstructionError::Overflow {
                    op: IntInstruction::Mod
                }
                .into()
            );
            assert!(result.is_recoverable());
            let top_int = result.state().stack::<PushInteger>().top().unwrap();
            prop_assert_eq!(*top_int, x);
        }
    }

    #[test]
    fn inc_does_not_crash(x in proptest::num::i64::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        let _ = IntInstruction::Inc.perform(state);
    }

    #[test]
    fn int_ops_do_not_crash(
            instr in proptest::sample::select(all_instructions()),
            x in proptest::num::i64::ANY,
            y in proptest::num::i64::ANY,
            b in proptest::bool::ANY) {
        let mut state = PushState::builder()
            .with_max_stack_size(100)
            .with_program([])
            .unwrap()
            .build();
        state.stack_mut::<PushInteger>().push(y).unwrap();
        state.stack_mut::<PushInteger>().push(x).unwrap();
        state.stack_mut::<bool>().push(b).unwrap();
        let _ = instr.perform(state);
    }
}