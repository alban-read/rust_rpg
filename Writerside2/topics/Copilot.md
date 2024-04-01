# Copilot

This is about my ongoing experience with Copilot (the gitHub coding assistant.)
I am writing a small project in rust, I have two key disadvantages which are:-

- I do not know rust.
- I have post-covid persistent brain fog, and focussing/attention difficulties.

So this is a good test of how useful an assistant Copilot can be.

Additionally, to Copilot, I am using JetBrains RustRover and the early preview of 
Jetbrains WriterSide.

Both of these are highly assistive tools, and they work well together.

The Copilot extension integrates with RustRover, providing completion, code creation, and chat features.

## Terminology

I will speak about Copilot as if it was a human assistant, however it is a computational model, it is not thinking in the same sense that we do, even if it seems to.



## Chat features

The chat features, allow you to discuss issues like the design and organization of the program.

You can describe objects, and discuss if they should be mutable.

Copilot will recommend if they should be mutable, and why.

This is a very useful feature, as it can be hard to know if an object should be mutable or not.

## Design and organization

Copilot will recommend quite high level organizational patterns.

As well as writing complete functions for you, it will recommend how to organize the functions.

Copilot can also explain and document the way a function works in great detail.

You can ask Copilot about language features, and it will explain them to you.

This makes Copilot a much better way to learn a new language than most other solutions.

I think language learning labs, and introductory text books, are probably on the way out.



## Completion and code generation.

As you type Copilot will make suggestion, if you type in a comment, copilot will generate complete solutions to the comment.

Copilot will generate complete, sometimes quite complex functions.

## Benefits of rust with copilot

The rust compiler, is horribly, annoyingly strict, and generates very clear error messages when mistakes are made.
Since rust is very strongly typed, it also allows the editor itself to detect lots of errors.
Due to the strong typing, code completion is naturally very good, and I think this makes it easier for the Copilot to constrain
the choices it makes as it recommends what should be typed next.

## Code correctness and trust

One thing I find hard to trust, when creating code I only partially understand, is code correctness.
Is the code correct?

Rust has an excellent testing, and test coverage workflow.
Because rust is extremely strict, I find it essential to write tests, Copilot is very good at writing tests for us.

The tests do sometimes fail, and the code needs to be corrected to ensure they pass.

This makes development, even in a language with a notoriously slow compiler seem very interactive.














