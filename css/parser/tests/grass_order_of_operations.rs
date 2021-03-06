#[macro_use]
mod grass_macros;

grass_test!(
    addition_then_division,
    "a {\n  color: 3+3/4;\n}\n",
    "a {\n  color: 3.75;\n}\n"
);
grass_test!(
    division_then_addition,
    "a {\n  color: 3/4 + 3;\n}\n",
    "a {\n  color: 3.75;\n}\n"
);
grass_test!(
    addition_then_multiplication,
    "a {\n  color: 4 + 2 * 3;\n}\n",
    "a {\n  color: 10;\n}\n"
);
grass_test!(
    multiplication_then_addition,
    "a {\n  color: 4 * 2 + 3;\n}\n",
    "a {\n  color: 11;\n}\n"
);
grass_test!(
    comparison,
    "a {\n  color: 1 < 1 and 1 < 1;\n}\n",
    "a {\n  color: false;\n}\n"
);
grass_test!(
    equals_then_or,
    "a {\n  color: a or b==c;\n}\n",
    "a {\n  color: a;\n}\n"
);
grass_test!(
    not_equals_then_or,
    "a {\n  color: a or b !=c;\n}\n",
    "a {\n  color: a;\n}\n"
);
grass_test!(
    leftmost_is_evaluated_first_when_same_precedence,
    "a {\n  color: 1 / 2 * 1em;\n}\n",
    "a {\n  color: 0.5em;\n}\n"
);
