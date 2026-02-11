import { useState } from 'react';
import { ProjectList } from './features/dashboard/ProjectList';
import { NewProjectWizard } from './features/onboarding/NewProjectWizard';
import './styles/globals.css';

function App() {
  const [showWizard, setShowWizard] = useState(false);
  const [refreshToken, setRefreshToken] = useState(0);
  const [activeProjectPath, setActiveProjectPath] = useState<string | null>(null);

  const handleWizardSuccess = (projectPath: string) => {
    setShowWizard(false);
    setRefreshToken((prev) => prev + 1);
    setActiveProjectPath(projectPath);
  };

  return (
    <div className="h-screen">
      <ProjectList
        refreshToken={refreshToken}
        onOpenProject={setActiveProjectPath}
        onCreateNew={() => setShowWizard(true)}
      />

      {showWizard && (
        <NewProjectWizard onClose={() => setShowWizard(false)} onSuccess={handleWizardSuccess} />
      )}

      {activeProjectPath && (
        <div className="pointer-events-none fixed bottom-4 right-4 rounded-lg bg-black/80 px-3 py-2 text-sm text-white shadow-lg">
          当前项目: {activeProjectPath}
        </div>
      )}
    </div>
  );
}

export default App;
