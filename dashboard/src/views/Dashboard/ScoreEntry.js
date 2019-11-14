import React, { Component } from 'react';
import {            
  } from 'reactstrap';


class ScoreEntry extends Component {
    constructor(props) {
		super(props);		
    }
    
	onUpdate = () => {
		this.setState({});
	}

	componentDidMount() {
		this.props.score.on('change', this.onUpdate);
	}

	componentWillUnmount() {
		this.props.score.off('change', this.onUpdate);
    }
    render() {
		return (
            <tr>
                <td>{this.props.score.player}</td>
                <td>{this.props.score.amount}</td>
            </tr>            
        )
    }
}

export default ScoreEntry;
