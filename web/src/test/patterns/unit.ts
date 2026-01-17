/**
 * Unit test patterns and conventions for TypeScript code.
 *
 * ## Naming Convention
 * - describe: Component or function name
 * - it: "should <expected behavior> when <condition>"
 *
 * ## Structure
 * - Group by component/function
 * - Nest by feature or scenario
 * - Use beforeEach for common setup
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// ============================================
// Pattern: Test Data Builder
// ============================================

export interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
  createdAt: Date;
}

export class UserBuilder {
  private data: Partial<User> = {};

  static create(): UserBuilder {
    return new UserBuilder();
  }

  withId(id: string): this {
    this.data.id = id;
    return this;
  }

  withName(name: string): this {
    this.data.name = name;
    return this;
  }

  withEmail(email: string): this {
    this.data.email = email;
    return this;
  }

  withRole(role: User['role']): this {
    this.data.role = role;
    return this;
  }

  asAdmin(): this {
    return this.withRole('admin');
  }

  asGuest(): this {
    return this.withRole('guest');
  }

  build(): User {
    return {
      id: this.data.id ?? `user-${Date.now()}`,
      name: this.data.name ?? 'Test User',
      email: this.data.email ?? 'test@example.com',
      role: this.data.role ?? 'user',
      createdAt: new Date(),
    };
  }
}

// ============================================
// Example Tests Using Patterns
// ============================================

describe('UserBuilder', () => {
  describe('default values', () => {
    it('should create user with sensible defaults', () => {
      // Arrange & Act
      const user = UserBuilder.create().build();

      // Assert
      expect(user.name).toBe('Test User');
      expect(user.email).toBe('test@example.com');
      expect(user.role).toBe('user');
    });

    it('should generate unique IDs', () => {
      const user1 = UserBuilder.create().build();
      const user2 = UserBuilder.create().build();

      expect(user1.id).not.toBe(user2.id);
    });
  });

  describe('role handling', () => {
    it.each([
      ['admin', 'admin'],
      ['user', 'user'],
      ['guest', 'guest'],
    ] as const)('should accept %s role', (role, expected) => {
      const user = UserBuilder.create().withRole(role).build();
      expect(user.role).toBe(expected);
    });

    it('should have admin helper', () => {
      const user = UserBuilder.create().asAdmin().build();
      expect(user.role).toBe('admin');
    });

    it('should have guest helper', () => {
      const user = UserBuilder.create().asGuest().build();
      expect(user.role).toBe('guest');
    });
  });

  describe('chaining', () => {
    it('should support method chaining', () => {
      const user = UserBuilder.create()
        .withId('custom-id')
        .withName('Jane Doe')
        .withEmail('jane@example.com')
        .asAdmin()
        .build();

      expect(user).toEqual(
        expect.objectContaining({
          id: 'custom-id',
          name: 'Jane Doe',
          email: 'jane@example.com',
          role: 'admin',
        })
      );
    });
  });
});

// ============================================
// Pattern: Testing Async Functions
// ============================================

async function fetchUserById(id: string): Promise<User | null> {
  // Simulated async operation
  await new Promise(resolve => setTimeout(resolve, 10));
  if (id === 'not-found') return null;
  return UserBuilder.create().withId(id).build();
}

describe('fetchUserById', () => {
  it('should return user when found', async () => {
    const user = await fetchUserById('test-id');

    expect(user).not.toBeNull();
    expect(user?.id).toBe('test-id');
  });

  it('should return null when not found', async () => {
    const user = await fetchUserById('not-found');
    expect(user).toBeNull();
  });
});

// ============================================
// Pattern: Testing with Mocks
// ============================================

interface ApiClient {
  get<T>(url: string): Promise<T>;
}

class UserService {
  constructor(private api: ApiClient) {}

  async getUser(id: string): Promise<User | null> {
    try {
      return await this.api.get<User>(`/users/${id}`);
    } catch {
      return null;
    }
  }
}

describe('UserService', () => {
  let mockApi: ApiClient;
  let service: UserService;

  beforeEach(() => {
    mockApi = {
      get: vi.fn(),
    };
    service = new UserService(mockApi);
  });

  it('should call API with correct URL', async () => {
    const mockUser = UserBuilder.create().withId('123').build();
    vi.mocked(mockApi.get).mockResolvedValue(mockUser);

    await service.getUser('123');

    expect(mockApi.get).toHaveBeenCalledWith('/users/123');
  });

  it('should return user from API', async () => {
    const mockUser = UserBuilder.create().withId('123').build();
    vi.mocked(mockApi.get).mockResolvedValue(mockUser);

    const result = await service.getUser('123');

    expect(result).toEqual(mockUser);
  });

  it('should return null on API error', async () => {
    vi.mocked(mockApi.get).mockRejectedValue(new Error('Network error'));

    const result = await service.getUser('123');

    expect(result).toBeNull();
  });
});