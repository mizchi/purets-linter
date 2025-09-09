// This should pass: one exported function with no side effects
interface User {
  id: string;
  name: string;
}

type UserWithAge = User & { age: number };

export function processUser(user: User): UserWithAge {
  return {
    ...user,
    age: 25
  };
}