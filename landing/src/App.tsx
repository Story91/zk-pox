import Hero from './components/Hero'
import HowItWorks from './components/HowItWorks'
import UserStory from './components/UserStory'
import Privacy from './components/Privacy'
import ProofTypes from './components/ProofTypes'
import AntiSpoof from './components/AntiSpoof'
import Architecture from './components/Architecture'
import Comparison from './components/Comparison'
import TechStack from './components/TechStack'
import Footer from './components/Footer'

export default function App() {
  return (
    <main className="min-h-screen">
      <Hero />
      <HowItWorks />
      <UserStory />
      <Privacy />
      <ProofTypes />
      <AntiSpoof />
      <Architecture />
      <Comparison />
      <TechStack />
      <Footer />
    </main>
  )
}
