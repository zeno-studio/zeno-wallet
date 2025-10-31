# Zeno Wallet 路由系统增强说明

## 新增功能

### 1. 页面切换动画支持
- 添加了四种过渡效果：`slide-left`、`slide-right`、`fade` 和 `none`
- 在页面切换时自动应用相应的 CSS 类
- 支持自定义过渡效果

### 2. 路由守卫功能
- 支持异步路由守卫
- 可用于权限验证、表单确认等场景
- 提供便捷的守卫管理方法

### 3. 增强的导航控制
- 所有导航方法现在都支持指定过渡效果
- 添加了带守卫的导航方法
- 改进了历史记录管理

### 4. 其他改进
- 添加了路由状态重置功能
- 增强了类型安全性
- 提供了更好的错误处理

## 使用示例

### 基本导航
```typescript
// 带动画的页面切换
page.push('settings', {}, 'slide-left');
page.pop('slide-right');

// 带参数的导航
page.push('export', { data: 'some-value' }, 'fade');
```

### 路由守卫
```typescript
// 添加认证守卫
const authGuard = async (to: Page, from: Page): Promise<boolean> => {
  const isAuthenticated = !!localStorage.getItem('authToken');
  if (!isAuthenticated && to === 'settings') {
    page.go('sign');
    return false;
  }
  return true;
};

page.addGuard(authGuard);
```

### 带守卫的导航
```typescript
// 使用守卫进行导航
await page.guardedPush('settings', {}, 'slide-left');
```