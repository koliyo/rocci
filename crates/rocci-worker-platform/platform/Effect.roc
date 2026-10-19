## Small ordinary Roc effect surface used to qualify the Worker ABI.
Effect := [].{
    Resource :: Box(U64)

    echo! : Str => Str
    delay! : U64 => Try(U64, [Rejected])
    resource! : U64 => Resource
    resource_value! : Resource => U64
}
