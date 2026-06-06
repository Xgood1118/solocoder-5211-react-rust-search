---
title: React Hooks 最佳实践
author: 李四
tags: react,前端,hooks
date: 2026-02-20T14:30:00Z
---

# React Hooks 最佳实践

React Hooks 自 16.8 版本引入以来，彻底改变了我们编写 React 组件的方式。本文总结一些实用的最佳实践。

## useState 使用技巧

### 1. 惰性初始化

当初始 state 需要复杂计算时，使用函数式初始化：

```javascript
const [data, setData] = useState(() => {
  return expensiveComputation(props.id);
});
```

### 2. 函数式更新

当新 state 依赖于旧 state 时：

```javascript
setCount(prev => prev + 1);
```

## useEffect 依赖项

### 正确处理依赖

- 只把真正依赖的值放到依赖数组中
- 使用 `useCallback` 和 `useMemo` 优化
- 考虑使用 `useRef` 保存最新值

### 清理副作用

```javascript
useEffect(() => {
  const subscription = props.source.subscribe();
  return () => {
    subscription.unsubscribe();
  };
}, [props.source]);
```

## 自定义 Hook

将可复用的逻辑抽取为自定义 Hook：

```javascript
function useLocalStorage(key, initialValue) {
  const [value, setValue] = useState(() => {
    const saved = localStorage.getItem(key);
    return saved ? JSON.parse(saved) : initialValue;
  });

  useEffect(() => {
    localStorage.setItem(key, JSON.stringify(value));
  }, [key, value]);

  return [value, setValue];
}
```

## 性能优化

1. 使用 `useMemo` 缓存计算结果
2. 使用 `useCallback` 缓存函数引用
3. 避免在渲染中创建新对象
4. 合理使用 `React.memo`

## 常见陷阱

- 不要在条件判断中调用 Hook
- 不要在循环中调用 Hook
- 不要在嵌套函数中调用 Hook
- 依赖项要完整但不要冗余

遵循这些最佳实践，可以让你的 React 代码更清晰、更高效。
