// This file contains correct pure TypeScript code

type User = {
  readonly id: string;
  readonly name: string;
  readonly email: string;
};

export default function createUser(
  id: string, 
  name: string, 
  email: string
): User {
  return { id, name, email };
}