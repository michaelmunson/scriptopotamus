# Scriptopotamus Vision

## Overview
* Scriptopotamus is a psuedo-language that compiles to bash
* It is meant to clean up some of the nasty bash syntax, and also help enforce typing etc.
* It is written in pure bash

## Compiler
* The scriptopotamus compiler is loosely inspired by rust
* **Declaring Variables**
    * when declaring a variable with a type, scriptopotamus identifies all subsequent assignments to that variable and inserts a type check

