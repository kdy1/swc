function* test() {
    while (!False()) {
        // execute this line
        while (!False()) {
            // execute this line
            break;
        }
        // execute this line
        if (False()) {
            // NOT execute this line
            break;
        }

        // execute this line
        yield "correct";
        return;
    }

    // NOT execute this line
    yield "wrong";
    return;
}

function False() {
    return false;
}

for (const result of test()) {
    console.log(result);
}