# rlox


https://www.craftinginterpreters.com/


### Scanning (lexing, lexical analysis)

lexer(stream[char]) -> stream[token]


### Parsing

Syntax gets grammar

parser(stream[token]) -> abstract syntax tree (ASTs)


### Static analysis

First step usually binding/resolution.

For each identifier, find defnition and couple together.

Store analysis as attributes on AST or in a lookup table off the side (symbol table).
Or transform AST into another data structure.

^^ is considered front end


### Intermediate representations (IR)

https://en.wikipedia.org/wiki/Control-flow_graph

https://en.wikipedia.org/wiki/Static_single-assignment_form

https://en.wikipedia.org/wiki/Continuation-passing_style

https://en.wikipedia.org/wiki/Three-address_code


### Optimization

constant folding is based


### Code generation

(back end) primitive assembly-like instructions, not source code, machine code

Martin Richards and Niklaus Wirth (gigachads) p-code (portable code) => bytecode


---

Tree-walk interpreters == interpreter

source-to-source compiler, transcompiler == transpiler

jit == just-in-time compilation


### The Lox Language

```lox
print "Kebab pizza!";

true;
false;
1234;   // An integer.
12.24;  // A decimal number.

"I am a string";
"";
"123";  // This is a string.

nil;
```

If built-in data types and their literals are atoms, then **expressions** must be the molecules.

```lox
add + me;
subtract - me;
multiply - me;
divide / me;
```

infix operators (because operator inbetween the atoms)

```lox
less < than;
lessThan <= orEqual;
greater > than;
greaterThan >= orEqual;

1 == 2;
"cat" != "dog";  // true.

314 == "pi";  // false.
123 == "123;  // false.

!true;  // false.
!false; // true.

true and false; // false.
true and true;  // true.

false or false; // false.
true or false;  // true.
```

Where an expression’s main job is to produce a value, a statement’s job is to produce an effect.

";" is an expression statement.

```lox
{
  print "One statement.";
  print "Two statements.";
}
```


### Variables

```lox
var imAVariable = "here is my value";
var iAmNil;

var breakfast = "bagels";
print breakfast; // "bagels".
breakfast = "beignets";
print breakfast; // "beignets".
```


### Control Flow

```lox
if (condition) {
  print "yes";
} else {
  print "no";
}

var a = 1;
while (a < 10) {
  print a;
  a = a + 1;
}

for (var a = 1; a < 10; a = a + 1) {
  print a;
}
```

### Functions

```lox
makeBreakfast(bacon, eggs, toast);

fun printSum(a, b) {
  print a + b;
}
```

**Argument** is an actual value you pass to a function. A **parameter** is a variable that holds the
value of the argument inside the body of the function.


### Closures

Functions are first class, they are real values.

```lox
fun addPair(a, b) {
  return a + b;
}

fun identity(a) {
  return a;
}

print identity(addPair)(1, 2); // Prints "3".

fun outerFunction() {
  fun localFunction() {
    print "I'm local!";
  }

  localFunction();
}

fun returnFunction() {
  var outside = "outside";

  fun inner() {
    print outside;
  }

  return inner;
}

var fn = returnFunction();
fn();
```


### Classes

```lox
class Breakfast {
  cook() {
    print "Eggs a-fryin'!";
  }

  serve(who) {
    print "Enjoy your breakfast, " + who + ".";
  }
}

// Store it in variables.
var someVariable = Breakfast;

// Pass it to functions.
someFunction(Breakfast);

var breakfast = Breakfast();
print breakfast; // "Breakfast instance".

breakfast.meat = "sausage";
breakfast.bread = "sourdough";

class Breakfast {
  serve(who) {
    print "Enjoy your " + this.meat + " and " +
        this.bread + ", " + who + ".";
  }

  // ...
}

class Breakfast {
  init(meat, bread) {
    this.meat = meat;
    this.bread = bread;
  }

  // ...
}

var baconAndToast = Breakfast("bacon", "toast");
baconAndToast.serve("Dear Reader");
// "Enjoy your bacon and toast, Dear Reader."
```


### Inheritance

```lox
class Brunch < Breakfast {
  drink() {
    print "How about a Bloody Mary?";
  }
}

var benedict = Brunch("ham", "English muffin");
benedict.serve("Noble Reader");

class Brunch < Breakfast {
  init(meat, bread, drink) {
    super.init(meat, bread);
    this.drink = drink;
  }
}
```
