import React from 'react';
import './styles/global.css';

export default function App({ Component }: { Component: React.ComponentType }) {
  return <Component />;
}
