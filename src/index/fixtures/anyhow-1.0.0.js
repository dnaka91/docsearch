var N=null,E="",T="t",U="u",searchIndex={};
var R=["backtrace","option","context","Wrap the error value with additional context.","result","error","anyhow","try_from","try_into","borrow_mut","formatter"];

searchIndex["anyhow"]={"doc":"This library provides [`anyhow::Error`][Error], a trait…","i":[[3,"Chain",R[6],"Iterator of a chain of source errors.",N,N],[3,"Error",E,"The `Error` type, a wrapper around a dynamic error type.",N,N],[11,"new",E,"Create a new error object from any error type.",0,[[["e"]],["self"]]],[11,R[2],E,R[3],0,[[["c"]],["self"]]],[11,R[0],E,"Get the backtrace for this Error.",0,[[["self"]],[R[0]]]],[11,"chain",E,"An iterator of the chain of source errors contained by…",0,[[["self"]],["chain"]]],[11,"root_cause",E,"The lowest level cause of this error — this error's…",0,[[["self"]],["stderror"]]],[11,"is",E,"Returns `true` if `E` is the type wrapped by this error…",0,[[["self"]],["bool"]]],[11,"downcast",E,"Attempt to downcast the error object to a concrete type.",0,[[],[R[4]]]],[11,"downcast_ref",E,"Downcast this error object by reference.",0,[[["self"]],[[R[1]],["e"]]]],[11,"downcast_mut",E,"Downcast this error object by mutable reference.",0,[[["self"]],[["e"],[R[1]]]]],[6,"Result",E,"`Result<T, Error>`",N,N],[8,"Context",E,"Provides the `context` method for `Result`.",N,N],[10,R[2],E,R[3],1,[[["c"]],[[R[5]],[R[4],[R[5]]]]]],[10,"with_context",E,"Wrap the error value with additional context that is…",1,[[["f"]],[[R[5]],[R[4],[R[5]]]]]],[14,"bail",E,"Return early with an error.",N,N],[14,R[6],E,"Construct an ad-hoc error from a string.",N,N],[11,"into",E,E,2,[[],[U]]],[11,"into_iter",E,E,2,[[],["i"]]],[11,"from",E,E,2,[[[T]],[T]]],[11,R[7],E,E,2,[[[U]],[R[4]]]],[11,R[8],E,E,2,[[],[R[4]]]],[11,R[9],E,E,2,[[["self"]],[T]]],[11,"borrow",E,E,2,[[["self"]],[T]]],[11,"type_id",E,E,2,[[["self"]],["typeid"]]],[11,"into",E,E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,"to_string",E,E,0,[[["self"]],["string"]]],[11,R[7],E,E,0,[[[U]],[R[4]]]],[11,R[8],E,E,0,[[],[R[4]]]],[11,R[9],E,E,0,[[["self"]],[T]]],[11,"borrow",E,E,0,[[["self"]],[T]]],[11,"type_id",E,E,0,[[["self"]],["typeid"]]],[11,"drop",E,E,0,[[["self"]]]],[11,"from",E,E,0,[[["e"]],["self"]]],[11,"next",E,E,2,[[["self"]],[R[1]]]],[11,"deref_mut",E,E,0,[[["self"]]]],[11,"deref",E,E,0,[[["self"]]]],[11,"fmt",E,E,0,[[["self"],[R[10]]],[R[4]]]],[11,"fmt",E,E,0,[[["self"],[R[10]]],[R[4]]]]],"p":[[3,"Error"],[8,"Context"],[3,"Chain"]]};
initSearch(searchIndex);addSearchOptions(searchIndex);