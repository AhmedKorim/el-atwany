const Express = require( 'express' );
const cors = require('cors');
const path = require('path')
const app = Express();


app.use(cors())

app.use( Express.static( path.join(__dirname , '..' ) ) );
app.use((req,res,next) =>{
    res.send('handled')
    next()
})
app.listen( 8081 );


