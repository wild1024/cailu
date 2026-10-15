/**
 * TDesign 主题色取值。
 *
 * `t-tag` / `t-badge` / `t-link` 等组件的 `theme` 属性在类型上是**字面量联合**，
 * 因此各模块中「取值 → 主题色」的映射函数必须返回本类型（而非宽泛的 `string`），
 * 否则模板里 `:theme="xxxTheme(row)"` 会触发 TS2322。
 */
export type TagTheme = 'default' | 'primary' | 'success' | 'warning' | 'danger';
