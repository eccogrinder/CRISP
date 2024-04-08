import React, { useState } from 'react'
import CardContent from '../../components/Cards/CardContent'
import CircleIcon from '../../assets/icons/caretCircle.svg'
import CodeTextDisplay from '../../components/CodeTextDisplay'
import VotesBadge from '../../components/VotesBadge'
import PollCardResult from '../../components/Cards/PollCardResult'
import { markWinner } from '../../utils/methods'
import PastPollSection from '../Landing/components/PastPoll'

const POLL = {
  id: 1,
  totalVotes: 451,
  date: 'March 25, 2024',
  options: [
    { id: 1, votes: 230, label: '🌮' },
    { id: 2, votes: 221, label: '🍕' },
  ],
}
const PollResult: React.FC = () => {
  const { totalVotes, date, options } = POLL
  const [showCode, setShowCode] = useState<boolean>(false)

  return (
    <div className='mb-28 flex w-screen flex-col items-center justify-center space-y-28'>
      <div className='my-28 flex w-screen flex-col items-center justify-center space-y-12'>
        <div className='flex flex-col items-center justify-center space-y-6'>
          <div className='space-y-2 text-center'>
            <p className='text-sm font-extrabold uppercase'>daily poll</p>
            <h1 className='text-h1 font-bold text-twilight-blue-900'>Results for most recent poll</h1>
            <p className=' text-2xl font-bold'>{date}</p>
          </div>

          <VotesBadge totalVotes={totalVotes} />
        </div>
        <div className='flex justify-center space-x-12'>
          <PollCardResult results={markWinner(options)} totalVotes={totalVotes} isResult width={288} height={288} />
        </div>
      </div>

      <CardContent>
        <div className='space-y-4'>
          <p className='text-base font-extrabold uppercase text-twilight-blue-500'>WHAT JUST HAPPENED?</p>
          <div className='space-y-2'>
            <p className='text-xl leading-8 text-twilight-blue-900'>
              After casting your vote, CRISP securely processed your selection using a blend of Fully Homomorphic Encryption (FHE),
              threshold cryptography, and zero-knowledge proofs (ZKPs), without revealing your identity or choice. Your vote was encrypted
              and anonymously aggregated with others, ensuring the integrity of the voting process while strictly maintaining
              confidentiality. The protocol's advanced cryptographic techniques guarantee that your vote contributes to the final outcome
              without any risk of privacy breaches or undue influence.
            </p>
            <div className='flex cursor-pointer items-center space-x-2' onClick={() => setShowCode(!showCode)}>
              <p className='text-green-light underline'>See what&apos;s happening under the hood</p>
              <img src={CircleIcon} className='h-[18] w-[18]' />
            </div>
            {showCode && <CodeTextDisplay />}
          </div>
        </div>
        <div className='space-y-4'>
          <p className='text-base font-extrabold uppercase text-twilight-blue-500'>WHAT DOES THIS MEAN?</p>
          <p className='text-xl leading-8 text-twilight-blue-900'>
            Your participation has directly contributed to a transparent and fair decision-making process, showcasing the power of
            privacy-preserving technology in governance and beyond. The use of CRISP in this vote represents a significant step towards
            secure, anonymous, and tamper-proof digital elections and polls. This innovation ensures that every vote counts equally while
            safeguarding against the risks of fraud and collusion, enhancing the reliability and trustworthiness of digital decision-making
            platforms.
          </p>
        </div>
      </CardContent>
      <PastPollSection customClass='' customLabel='Historic polls' />
    </div>
  )
}

export default PollResult
