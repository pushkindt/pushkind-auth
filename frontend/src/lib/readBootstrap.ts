export function readBootstrap<T>(elementId: string): T {
  const element = document.getElementById(elementId);

  if (!element) {
    throw new Error(`Missing bootstrap element: ${elementId}`);
  }

  const content = element.textContent;

  if (!content) {
    throw new Error(`Bootstrap element ${elementId} was empty`);
  }

  return JSON.parse(content) as T;
}
