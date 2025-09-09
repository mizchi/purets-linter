use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pure_ts::Linter;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;

const SMALL_CODE: &str = r#"
function add(a: number, b: number): number {
    return a + b;
}

const result = add(1, 2);
export { add };
"#;

const MEDIUM_CODE: &str = r#"
interface User {
    id: string;
    name: string;
    email: string;
}

class UserManager {
    private users: Map<string, User> = new Map();
    
    addUser(user: User): void {
        this.users.set(user.id, user);
    }
    
    getUser(id: string): User | undefined {
        return this.users.get(id);
    }
    
    removeUser(id: string): boolean {
        return this.users.delete(id);
    }
    
    getAllUsers(): User[] {
        return Array.from(this.users.values());
    }
}

export function createUserManager(): UserManager {
    return new UserManager();
}

function validateEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}

export { validateEmail };
"#;

const LARGE_CODE: &str = r#"
import { Result, ok, err } from 'neverthrow';

interface Product {
    id: string;
    name: string;
    price: number;
    category: string;
    inStock: boolean;
}

interface Order {
    id: string;
    userId: string;
    products: OrderItem[];
    total: number;
    status: OrderStatus;
}

interface OrderItem {
    productId: string;
    quantity: number;
    price: number;
}

type OrderStatus = 'pending' | 'processing' | 'shipped' | 'delivered' | 'cancelled';

class OrderService {
    private orders: Map<string, Order> = new Map();
    private products: Map<string, Product> = new Map();
    
    createOrder(userId: string, items: OrderItem[]): Result<Order, string> {
        if (items.length === 0) {
            return err('Order must contain at least one item');
        }
        
        let total = 0;
        for (const item of items) {
            const product = this.products.get(item.productId);
            if (!product) {
                return err(`Product ${item.productId} not found`);
            }
            if (!product.inStock) {
                return err(`Product ${product.name} is out of stock`);
            }
            total += item.price * item.quantity;
        }
        
        const order: Order = {
            id: this.generateOrderId(),
            userId,
            products: items,
            total,
            status: 'pending'
        };
        
        this.orders.set(order.id, order);
        return ok(order);
    }
    
    updateOrderStatus(orderId: string, status: OrderStatus): Result<Order, string> {
        const order = this.orders.get(orderId);
        if (!order) {
            return err(`Order ${orderId} not found`);
        }
        
        order.status = status;
        return ok(order);
    }
    
    getOrder(orderId: string): Result<Order, string> {
        const order = this.orders.get(orderId);
        if (!order) {
            return err(`Order ${orderId} not found`);
        }
        return ok(order);
    }
    
    getUserOrders(userId: string): Order[] {
        const userOrders: Order[] = [];
        for (const order of this.orders.values()) {
            if (order.userId === userId) {
                userOrders.push(order);
            }
        }
        return userOrders;
    }
    
    private generateOrderId(): string {
        return `ORD-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    }
}

export { OrderService, Product, Order, OrderItem, OrderStatus };
"#;

fn benchmark_linter(c: &mut Criterion) {
    let mut group = c.benchmark_group("linter");
    
    for (name, code) in &[
        ("small", SMALL_CODE),
        ("medium", MEDIUM_CODE),
        ("large", LARGE_CODE),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(name), code, |b, code| {
            b.iter(|| {
                let allocator = Allocator::default();
                let source_type = SourceType::from_path("test.ts").unwrap();
                let ret = Parser::new(&allocator, code, source_type).parse();
                
                let mut linter = Linter::new(Path::new("test.ts"), code, false);
                linter.check_program(black_box(&ret.program));
            });
        });
    }
    
    group.finish();
}

fn benchmark_parse_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_only");
    
    for (name, code) in &[
        ("small", SMALL_CODE),
        ("medium", MEDIUM_CODE),
        ("large", LARGE_CODE),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(name), code, |b, code| {
            b.iter(|| {
                let allocator = Allocator::default();
                let source_type = SourceType::from_path("test.ts").unwrap();
                let _ret = Parser::new(&allocator, code, source_type).parse();
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_linter, benchmark_parse_only);
criterion_main!(benches);