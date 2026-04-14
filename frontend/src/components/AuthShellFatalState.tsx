import { ShellFatalState } from "../../../../pushkind-common/frontend/src/ShellFatalState";

type AuthShellFatalStateProps = {
  message: string;
};

export function AuthShellFatalState({ message }: AuthShellFatalStateProps) {
  return (
    <ShellFatalState
      message={message}
      serviceLabel="pushkind-auth"
      title="Не удалось загрузить оболочку"
      shellClassName="container py-5"
      cardClassName="card mx-auto mt-4"
      eyebrowClassName="card-header text-muted fw-bold"
      titleClassName="card-title h4"
      messageClassName="card-text text-secondary mb-0"
    />
  );
}
