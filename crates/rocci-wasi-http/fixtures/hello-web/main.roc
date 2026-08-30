app [main!] { pf: platform "../../platform/main.roc" }

main! : {} => [Ok({}), Err([Exit(I32)])]
main! = |{}| {
    res : [Ok({}), Err([Exit(I32)])]
    res = Ok({})
    res
}
