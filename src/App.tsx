import { useState } from 'react'
import { loadingimg } from './assets/images'
import './App.css'

function App() {
  const [count, setCount] = useState(0)

  return (
    <>
    <h2>Image Classification & Detection</h2>
    <div className="card">
      <div className="container">
        <div className='input-file'>
          <input type="file" name="file" id="file" />
        </div>
        <div className='image-box'>
          <img src={loadingimg} alt="" className='image'/>
        </div>
        <div className='description'>
          <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. In erat nulla, 
            egestas at ipsum vitae, dictum pretium nibh. Fusce hendrerit.</p>
        </div>
        <div className='result'>
          <h2>Image Result</h2>
          <p>Image Size: </p>
          <p>object Detected: </p>
        </div>

          <button className='submit-btn'>Submit</button>

      </div>
    </div>
    </>
  )
}

export default App
