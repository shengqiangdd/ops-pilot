import { useEffect, useState, createContext, useContext } from 'react';
import { useNavigate } from 'react-router-dom';

type Role = 'admin' | 'operator' | 'viewer';

interface AuthContextValue {
  role: Role | null;
  hasRole: (required: Role) => boolean;
  canAccess: (requiredRoles: Role[]) => boolean;
}

const AuthContext = createContext<AuthContextValue>({
  role: null,
  hasRole: () => false,
  canAccess: () => false,
});

export const useAuth = () => useContext(AuthContext);

const ROLE_HIERARCHY: Record<Role, number> = {
  admin: 3,
  operator: 2,
  viewer: 1,
};

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [role, setRole] = useState<Role | null>(null);

  useEffect(() => {
    try {
      const stored = localStorage.getItem('opspilot-auth-role');
      if (stored && ['admin', 'operator', 'viewer'].includes(stored)) {
        setRole(stored as Role);
      }
    } catch { /* ignore */ }
  }, []);

  const hasRole = (required: Role) => {
    if (!role) return false;
    return ROLE_HIERARCHY[role] >= ROLE_HIERARCHY[required];
  };

  const canAccess = (requiredRoles: Role[]) => {
    if (!role) return false;
    return requiredRoles.some(r => ROLE_HIERARCHY[role] >= ROLE_HIERARCHY[r]);
  };

  return (
    <AuthContext.Provider value={{ role, hasRole, canAccess }}>
      {children}
    </AuthContext.Provider>
  );
}

interface AuthGuardProps {
  requiredRole?: Role;
  children: React.ReactNode;
}

export function AuthGuard({ requiredRole, children }: AuthGuardProps) {
  const { hasRole } = useAuth();
  const navigate = useNavigate();
  const [checked, setChecked] = useState(false);

  useEffect(() => {
    if (requiredRole && !hasRole(requiredRole)) {
      // Redirect to dashboard with error
      navigate('/dashboard', { state: { error: 'insufficient_permissions' } });
    } else {
      setChecked(true);
    }
  }, [requiredRole, hasRole, navigate]);

  if (!checked) return null;
  return <>{children}</>;
}

// Route configuration with required roles
export const ROUTE_ROLES: Record<string, Role[]> = {
  '/users': ['admin'],
  '/audit': ['admin'],
  '/hosts': ['operator', 'admin'],
  '/terminal': ['operator', 'admin'],
  '/vault': ['operator', 'admin'],
  '/cmdb': ['operator', 'admin'],
  '/cicd': ['operator', 'admin'],
  '/jobs': ['operator', 'admin'],
  '/monitor': ['viewer', 'operator', 'admin'],
  '/dashboard': ['viewer', 'operator', 'admin'],
  '/metrics': ['viewer', 'operator', 'admin'],
  '/topo': ['viewer', 'operator', 'admin'],
};
