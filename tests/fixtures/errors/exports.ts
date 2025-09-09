// Error: Various export issues

// Error: Named exports not allowed
export const value = 42;

// Error: Export const needs type
export const untypedValue = "test";

// Error: Export let not allowed
export let mutableValue = 100;

// Error: Re-exports not allowed
export * from './other-module';