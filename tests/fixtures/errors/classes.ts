// This file contains intentional errors for testing

// Error: Classes are not allowed
class User {
  private name: string;
  constructor(name: string) {
    this.name = name;
  }
}

// Error: Abstract classes are not allowed
abstract class BaseModel {
  abstract save(): void;
}